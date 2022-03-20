/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

mod auth;
pub mod filters;
mod view_mode;

pub use auth::Auth;
pub use view_mode::ViewMode;

use mailparse::ParsedMail;

use self::auth::GoogleAuthExt;
use self::filters::Filters;
use crate::auth::Google as GoogleAuth;
use crate::entry::Entry;
use crate::error::{Error, Result};
use crate::sink::Message;

const IMAP_PORT: u16 = 993;

pub struct Email {
	imap: String,
	email: String,
	auth: Auth,
	filters: Filters,
	view_mode: ViewMode,
	footer: Option<String>, // remove everything after this text, including itself, from the message
}

impl Email {
	#[must_use]
	pub fn with_password(
		imap: String,
		email: String,
		password: String,
		filters: Filters,
		view_mode: ViewMode,
		footer: Option<String>,
	) -> Self {
		Self {
			imap,
			email,
			auth: Auth::Password(password),
			filters,
			view_mode,
			footer,
		}
	}

	// FIXME: google oauth2 is only available to gmail.com, remove the imap field
	#[must_use]
	pub fn with_google_oauth2(
		imap: String,
		email: String,
		auth: GoogleAuth,
		filters: Filters,
		view_mode: ViewMode,
		footer: Option<String>,
	) -> Self {
		Self {
			imap,
			email,
			auth: Auth::GoogleAuth(auth),
			filters,
			view_mode,
			footer,
		}
	}

	/// Even though it's marked async, the fetching itself is not async yet
	/// It should be used with spawn_blocking probs
	/// TODO: make it async lol
	#[tracing::instrument(skip_all)]
	pub async fn get(&mut self) -> Result<Vec<Entry>> {
		tracing::debug!("Fetching emails");
		let client = imap::ClientBuilder::new(&self.imap, IMAP_PORT).rustls()?;

		let mut session = match &mut self.auth {
			Auth::GoogleAuth(auth) => {
				tracing::trace!("Logging in to IMAP with Google OAuth2");

				client
					.authenticate("XOAUTH2", &auth.as_imap_oauth2(&self.email).await?)
					// .map_err(|(e, _)| Error::EmailAuth(e))?
					.map_err(|(e, _)| Error::from(e))?
			}
			Auth::Password(password) => {
				tracing::warn!("Logging in to IMAP with a password, this is insecure");

				client
					.login(&self.email, password)
					// .map_err(|(e, _)| Error::EmailAuth(e))?
					.map_err(|(e, _)| Error::from(e))?
			}
		};

		session.examine("INBOX")?;

		let search_string = {
			let mut tmp = "UNSEEN ".to_string();

			if let Some(sender) = &self.filters.sender {
				tmp.push_str(&format!(r#"FROM "{sender}" "#));
			}

			if let Some(subjects) = &self.filters.subjects {
				for s in subjects {
					tmp.push_str(&format!(r#"SUBJECT "{s}" "#));
				}
			}

			if let Some(ex_subjects) = &self.filters.exclude_subjects {
				for exs in ex_subjects {
					tmp.push_str(&format!(r#"NOT SUBJECT "{exs}" "#));
				}
			}

			tmp.trim_end().to_string()
		};

		let mail_ids = session
			.uid_search(&search_string)?
			.into_iter()
			.map(|x| x.to_string())
			.collect::<Vec<_>>()
			.join(",");

		let unread_num = mail_ids.len();
		if unread_num > 0 {
			tracing::info!("Got {unread_num} unread filtered mails");
		} else {
			tracing::debug!(
				"All email for the search query have already been read, none remaining to send"
			);
		}

		if mail_ids.is_empty() {
			return Ok(Vec::new());
		}

		let mails = session.uid_fetch(&mail_ids, "BODY[]")?;
		session.logout()?;

		mails
			.iter()
			.filter(|x| x.body().is_some()) // TODO: properly handle error cases and don't just filter them out
			.rev()
			.map(|x| {
				self.parse(
					&mailparse::parse_mail(x.body().unwrap())?, // unwrap NOTE: temporary but it's safe for now because of the check above
					x.uid.unwrap().to_string(),
				)
			})
			.collect::<Result<Vec<Entry>>>()
	}

	fn parse(&self, mail: &ParsedMail, id: String) -> Result<Entry> {
		let subject = mail.headers.iter().find_map(|x| {
			if x.get_key_ref() == "Subject" {
				Some(x.get_value())
			} else {
				None
			}
		});

		let body = {
			let mut body = if mail.subparts.is_empty() {
				mail
			} else {
				mail.subparts
					.iter()
					.find(|x| x.ctype.mimetype == "text/plain")
					.unwrap_or(&mail.subparts[0])
			}
			.get_body()?;

			if let Some(footer) = self.footer.as_deref() {
				body.drain(body.find(footer).unwrap_or(body.len())..);
			}

			body
		};

		Ok(Entry {
			id,
			msg: Message {
				title: subject,
				body,
				link: None,
				media: None,
			},
		})
	}

	// FIXME: doesn't actually work for some reason
	pub(crate) async fn mark_as_read(&mut self, uid: &str) -> Result<()> {
		if let ViewMode::ReadOnly = self.view_mode {
			return Ok(());
		}

		let client = imap::ClientBuilder::new(&self.imap, IMAP_PORT).rustls()?;
		let mut session = match &mut self.auth {
			Auth::GoogleAuth(auth) => {
				tracing::trace!("Logging in to IMAP with Google OAuth2");

				client
					.authenticate("XOAUTH2", &auth.as_imap_oauth2(&self.email).await?)
					// .map_err(|(e, _)| Error::EmailAuth(e))?
					.map_err(|(e, _)| Error::from(e))?
			}
			Auth::Password(password) => {
				tracing::warn!("Logging in to IMAP with a password, this is insecure");

				client
					.login(&self.email, password)
					// .map_err(|(e, _)| Error::EmailAuth(e))?
					.map_err(|(e, _)| Error::from(e))?
			}
		};

		session.select("INBOX")?;

		match self.view_mode {
			ViewMode::MarkAsRead => {
				session.uid_store(uid, "+FLAGS.SILENT (\\Seen)")?;
				tracing::debug!("Marked email uid {uid} as read");
			}
			ViewMode::Delete => {
				session.uid_store(uid, "+FLAGS.SILENT (\\Deleted)")?;
				session.uid_expunge(uid)?;
				tracing::debug!("Deleted email uid {uid}");
			}
			ViewMode::ReadOnly => unreachable!(),
		};

		session.logout()?;

		Ok(())
	}
}

impl std::fmt::Debug for Email {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Email")
			// .field("name", &self.name)
			.field("imap", &self.imap)
			.field(
				"auth_type",
				match self.auth {
					Auth::Password(_) => &"password",
					Auth::GoogleAuth(_) => &"google_auth",
				},
			)
			.field("email", &self.email)
			.field("filters", &self.filters)
			.field("view_mode", &self.view_mode)
			.field("footer", &self.footer)
			.finish()
	}
}
