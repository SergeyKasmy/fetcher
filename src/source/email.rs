/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

mod auth;
mod view_mode;

pub use auth::Auth;
pub use view_mode::ViewMode;

use mailparse::ParsedMail;

use self::auth::GoogleAuthExt;
use crate::auth::GoogleAuth;
use crate::error::{Error, Result};
use crate::sink::Message;
use crate::source::Responce;

const IMAP_PORT: u16 = 993;

#[derive(Debug)]
pub struct Filters {
	pub sender: Option<String>,
	pub subjects: Option<Vec<String>>,
	pub exclude_subjects: Option<Vec<String>>,
}

pub struct Email {
	name: String,
	imap: String,
	email: String,
	auth: Auth,
	filters: Filters,
	view_mode: ViewMode,
	footer: Option<String>, // remove everything after this text, including itself, from the message
}

impl Email {
	#[tracing::instrument(skip(password))]
	pub fn with_password(
		name: String,
		imap: String,
		email: String,
		password: String,
		filters: Filters,
		view_mode: ViewMode,
		footer: Option<String>,
	) -> Self {
		tracing::info!("Creatng an Email provider");
		Self {
			name,
			imap,
			email,
			auth: Auth::Password(password),
			filters,
			view_mode,
			footer,
		}
	}

	#[tracing::instrument(skip(auth))]
	pub async fn with_google_oauth2(
		name: String,
		imap: String,
		email: String,
		auth: GoogleAuth,
		filters: Filters,
		view_mode: ViewMode,
		footer: Option<String>,
	) -> Result<Self> {
		tracing::info!("Creatng an Email provider");

		Ok(Self {
			name,
			imap,
			email,
			auth: Auth::GoogleAuth(auth),
			filters,
			view_mode,
			footer,
		})
	}

	/// Even though it's marked async, the fetching itself is not async yet
	/// It should be used with spawn_blocking probs
	#[tracing::instrument]
	pub async fn get(&mut self) -> Result<Vec<Responce>> {
		let client = imap::connect(
			(self.imap.as_str(), IMAP_PORT),
			&self.imap,
			&native_tls::TlsConnector::new().map_err(Error::Tls)?,
		)?;

		let mut session = match &mut self.auth {
			Auth::Password(password) => client
				.login(&self.email, password)
				.map_err(|(e, _)| Error::EmailAuth(e))?,
			Auth::GoogleAuth(auth) => client
				.authenticate("XOAUTH2", &auth.to_imap_oauth2(&self.email).await?)
				.map_err(|(e, _)| Error::EmailAuth(e))?,
		};

		match self.view_mode {
			ViewMode::ReadOnly => session.examine("INBOX"),
			ViewMode::MarkAsRead | ViewMode::Delete => session.select("INBOX"),
		}?;

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
			.uid_search(search_string)?
			.into_iter()
			.map(|x| x.to_string())
			.collect::<Vec<_>>()
			.join(",");

		if mail_ids.is_empty() {
			return Ok(Vec::new());
		}

		// TODO: reverse order
		let mails = session.uid_fetch(&mail_ids, "BODY[]")?;

		// TODO: handle sent messages separately
		// mb a callback with email UID after successful sending?
		if let ViewMode::Delete = self.view_mode {
			session.uid_store(&mail_ids, "+FLAGS.SILENT (\\Deleted)")?;
			session.uid_expunge(&mail_ids)?;
		}

		session.logout()?;
		tracing::info!("Got {amount} unread emails", amount = mails.len());

		mails
			.into_iter()
			.filter(|x| x.body().is_some()) // TODO: properly handle error cases and don't just filter them out
			.map(|x| {
				Ok(Responce {
					id: None,
					msg: Self::parse(
						mailparse::parse_mail(x.body().unwrap())?, // unwrap NOTE: temporary but it's safe for now because of the check above
						self.footer.as_deref(),
					)?,
				})
			})
			.collect::<Result<Vec<Responce>>>()
	}

	fn parse(mail: ParsedMail, remove_after: Option<&str>) -> Result<Message> {
		let (subject, body) = {
			let subject = mail.headers.iter().find_map(|x| {
				if x.get_key_ref() == "Subject" {
					Some(x.get_value())
				} else {
					None
				}
			});

			let mut body = if mail.subparts.is_empty() {
				&mail
			} else {
				mail.subparts
					.iter()
					.find(|x| x.ctype.mimetype == "text/plain")
					.unwrap_or(&mail.subparts[0])
			}
			.get_body()?;

			if let Some(remove_after) = remove_after {
				body.drain(body.find(remove_after).unwrap_or_else(|| body.len())..);
			}

			// TODO: replace upticks ` with teloxide::utils::html::escape_code

			// NOTE: emails often contain all kinds of html or other text which Telegram's HTML parser doesn't approve of
			// I dislike the need to add an extra dependency just for this simple task but you gotta do what you gotta do.
			// Hopefully I'll find a better way to escape everything though since I don't fear a possibility that it'll be
			// somehow harmful 'cause it doesn't consern me, only Telegram :P
			(subject, ammonia::clean(&body))
		};

		let text = match subject {
			Some(subject) => format!("{}\n\n{}", subject, body),
			None => body,
		};

		Ok(Message { text, media: None })
	}
}

impl std::fmt::Debug for Email {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Email")
			.field("name", &self.name)
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
