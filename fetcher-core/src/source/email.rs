/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A email source that uses IMAP to connect to an email server
//!
//! This module includes the [`Email`] source, the [`ViewMode`] enum, and the [`Filters`] struct

mod auth;
mod filters;
mod view_mode;

pub use auth::Auth;
pub use filters::Filters;
pub use view_mode::ViewMode;

use self::auth::GoogleAuthExt;
use super::{Fetch, MarkAsRead, Source};
use crate::{
	auth::Google as GoogleAuth,
	entry::Entry,
	error::Error,
	sink::Message,
	source::error::{EmailError, ImapError, SourceError},
};

use async_trait::async_trait;
use mailparse::ParsedMail;
use std::fmt::{Debug, Write as _};

const IMAP_PORT: u16 = 993;

/// Email source. Fetches an email's subject and body fields using IMAP
pub struct Email {
	/// IMAP server URL
	pub imap: String,

	/// Email address/IMAP login
	pub email: String,

	/// Authentication type
	pub auth: Auth,

	/// IMAP search filters
	pub filters: Filters,

	/// IMAP view mode, e.g. read only
	pub view_mode: ViewMode,
}

// I'd make that a function but the imap crate didn't want to agree with me
macro_rules! authenticate {
	($login:expr, $auth:expr, $client:expr) => {{
		let auth = $auth;

		match auth {
			Auth::GoogleAuth(auth) => {
				tracing::trace!("Logging in to IMAP with Google OAuth2");

				$client
					.authenticate(
						"XOAUTH2",
						&auth
							.as_imap_oauth2($login)
							.await
							.map_err(|e| ImapError::GoogleAuth(Box::new(e)))?,
					)
					.map_err(|(e, _)| ImapError::Auth(e))?
			}
			Auth::Password(password) => {
				tracing::warn!("Logging in to IMAP with a password, this is insecure");

				$client
					.login($login, password)
					.map_err(|(e, _)| ImapError::Auth(e))?
			}
		}
	}};
}

impl Email {
	/// Creates an [`Email`] source for use with Gmail that uses [`Google OAuth2`](`crate::auth::Google`) to authenticate
	#[must_use]
	pub fn with_google_oauth2(
		email: String,
		auth: GoogleAuth,
		filters: Filters,
		view_mode: ViewMode,
	) -> Self {
		Self {
			imap: "imap.gmail.com".to_owned(),
			email,
			auth: Auth::GoogleAuth(auth),
			filters,
			view_mode,
		}
	}

	/// Creates an [`Email`] source that uses a password to authenticate via IMAP
	#[must_use]
	pub fn with_password(
		imap: String,
		email: String,
		password: String,
		filters: Filters,
		view_mode: ViewMode,
	) -> Self {
		Self {
			imap,
			email,
			auth: Auth::Password(password),
			filters,
			view_mode,
		}
	}
}

#[async_trait]
impl Fetch for Email {
	/// Even though it's marked async, the fetching itself is not async yet
	/// It should be used with spawn_blocking probs
	/// TODO: make it async lol
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		self.fetch_impl().await.map_err(Into::into)
	}
}

#[async_trait]
impl MarkAsRead for Email {
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		self.mark_as_read_impl(id)
			.await
			.map_err(|e| Error::from(SourceError::from(EmailError::from(e))))
	}

	async fn set_read_only(&mut self) {
		self.view_mode = ViewMode::ReadOnly;
	}
}

impl Source for Email {}

impl Email {
	async fn fetch_impl(&mut self) -> Result<Vec<Entry>, EmailError> {
		tracing::debug!("Fetching emails");
		let client = imap::ClientBuilder::new(&self.imap, IMAP_PORT)
			.rustls()
			.map_err(ImapError::TlsInitFailed)?;

		let mut session = authenticate!(&self.email, &mut self.auth, client);

		session.examine("INBOX").map_err(ImapError::Other)?;

		let search_string = {
			let mut tmp = "UNSEEN ".to_string();

			if let Some(sender) = &self.filters.sender {
				let _ = write!(tmp, r#"FROM "{sender}" "#);
			}

			if let Some(subjects) = &self.filters.subjects {
				for s in subjects {
					let _ = write!(tmp, r#"SUBJECT "{s}" "#);
				}
			}

			if let Some(ex_subjects) = &self.filters.exclude_subjects {
				for exs in ex_subjects {
					let _ = write!(tmp, r#"NOT SUBJECT "{exs}" "#);
				}
			}

			tmp.trim_end().to_string()
		};

		let mail_ids = session
			.uid_search(&search_string)
			.map_err(ImapError::Other)?
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

		let mails = session
			.uid_fetch(&mail_ids, "BODY[]")
			.map_err(ImapError::Other)?;
		session.logout().map_err(ImapError::Other)?;

		mails
			.iter()
			.map(|x| {
				let body = x
					.body()
					.expect("Body should always be present because we explicitly requested it");

let uid =
					x.uid.expect("UIDs should always be present because we used uid_fetch(). The server probably doesn't support them which isn't something ~we~ support for now").to_string();

				parse(
					&mailparse::parse_mail(body)?,
					uid,
				)
			})
			.collect::<Result<Vec<Entry>, EmailError>>()
	}

	async fn mark_as_read_impl(&mut self, id: &str) -> Result<(), ImapError> {
		if let ViewMode::ReadOnly = self.view_mode {
			return Ok(());
		}

		let client = imap::ClientBuilder::new(&self.imap, IMAP_PORT)
			.rustls()
			.map_err(ImapError::TlsInitFailed)?;
		let mut session = authenticate!(&self.email, &mut self.auth, client);

		session.select("INBOX")?;

		match self.view_mode {
			ViewMode::MarkAsRead => {
				session.uid_store(id, "+FLAGS.SILENT (\\Seen)")?;
				tracing::debug!("Marked email uid {id} as read");
			}
			ViewMode::Delete => {
				session.uid_store(id, "+FLAGS.SILENT (\\Deleted)")?;
				session.uid_expunge(id)?;
				tracing::debug!("Deleted email uid {id}");
			}
			ViewMode::ReadOnly => unreachable!(),
		};

		session.logout()?;

		Ok(())
	}
}

fn parse(mail: &ParsedMail, id: String) -> Result<Entry, EmailError> {
	let subject = mail.headers.iter().find_map(|x| {
		if x.get_key_ref() == "Subject" {
			Some(x.get_value())
		} else {
			None
		}
	});

	let body = {
		if mail.subparts.is_empty() {
			mail
		} else {
			mail.subparts
				.iter()
				.find(|x| x.ctype.mimetype == "text/plain")
				.unwrap_or(&mail.subparts[0])
		}
		.get_body()?
	};

	Ok(Entry {
		id: Some(id),
		msg: Message {
			title: subject,
			body: Some(body),
			..Default::default()
		},
		..Default::default()
	})
}

impl Debug for Email {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Email")
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
			.finish()
	}
}
