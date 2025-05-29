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

pub use self::{auth::Auth, filters::Filters, view_mode::ViewMode};

use self::auth::GoogleAuthExt;
use super::{Fetch, MarkAsRead, Source};
use crate::{
	StaticStr,
	auth::{Google as GoogleAuth, google::GoogleOAuth2Error as GoogleAuthError},
	entry::{Entry, EntryId},
	sinks::message::Message,
};

use async_imap::{Client, Session};
use futures::{StreamExt, TryStreamExt};
use mailparse::ParsedMail;
use std::{
	fmt::{Debug, Write as _},
	io,
	sync::{Arc, LazyLock},
};
use tokio::net::TcpStream;
use tokio_rustls::{
	TlsConnector,
	client::TlsStream,
	rustls::{
		ClientConfig, RootCertStore,
		pki_types::{InvalidDnsNameError, ServerName},
	},
};

const IMAP_PORT: u16 = 993;

static TLS_CONNECTOR: LazyLock<TlsConnector> = LazyLock::new(|| {
	let mut root_cert_store = RootCertStore::empty();
	root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

	let config = ClientConfig::builder()
		.with_root_certificates(root_cert_store)
		.with_no_client_auth();

	TlsConnector::from(Arc::new(config))
});

/// Email source. Fetches an email's subject and body fields using IMAP
pub struct Email {
	/// IMAP server address
	pub imap_server: StaticStr,

	/// Email address/IMAP login
	pub email: StaticStr,

	/// Authentication type
	pub auth: Auth,

	/// IMAP search filters
	pub filters: Filters,

	/// IMAP view mode, e.g. read only
	pub view_mode: ViewMode,
}

#[expect(missing_docs, reason = "error message is self-documenting")]
//#[expect(clippy::large_enum_variant, reason = "the entire enum is already boxed one level above")]
#[derive(thiserror::Error, Debug)]
pub enum EmailError {
	#[error("IMAP connection error")]
	Imap(#[from] ImapError),

	#[error("Error parsing email")]
	Parse(#[from] mailparse::MailParseError),
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum ImapError {
	#[error("Failed to connect to the IMAP server")]
	ConnectionFailed(#[source] io::Error),

	#[error("Failed to get the domain name of the IMAP server")]
	InvalidImapServerAddress(#[from] InvalidDnsNameError),

	#[error(transparent)]
	GoogleOAuth2(#[from] GoogleAuthError),

	#[error("Authentication error")]
	Auth(#[source] async_imap::error::Error),

	#[error(transparent)]
	Other(#[from] async_imap::error::Error),
}

#[bon::bon]
impl Email {
	/// Creates an [`Email`] source that uses a password to authenticate via IMAP
	#[builder]
	#[must_use]
	pub fn new_generic(
		#[builder(into)] imap_server: StaticStr,
		#[builder(into)] email: StaticStr,
		#[builder(into)] password: StaticStr,
		filters: Filters,
		view_mode: ViewMode,
	) -> Self {
		Self {
			imap_server,
			email,
			auth: Auth::Password(password),
			filters,
			view_mode,
		}
	}

	/// Creates an [`Email`] source for use with Gmail that uses [`Google OAuth2`](`crate::auth::Google`) to authenticate
	#[builder]
	#[must_use]
	pub fn new_gmail(
		#[builder(into)] email: StaticStr,
		auth: GoogleAuth,
		filters: Filters,
		view_mode: ViewMode,
	) -> Self {
		Self {
			imap_server: "imap.gmail.com".into(),
			email,
			auth: Auth::GmailOAuth2(auth),
			filters,
			view_mode,
		}
	}
}

impl Fetch for Email {
	type Err = EmailError;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		// TODO: inline this fn
		self.fetch_impl().await
	}
}

impl MarkAsRead for Email {
	type Err = ImapError;

	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Self::Err> {
		// TODO: inline this fn
		self.mark_as_read_impl(id).await
	}

	async fn set_read_only(&mut self) {
		self.view_mode = ViewMode::ReadOnly;
	}
}

impl Source for Email {}

impl Email {
	async fn client(&self) -> Result<Client<TlsStream<TcpStream>>, ImapError> {
		tracing::trace!("Connecting to the IMAP server");

		let tcp_stream = TcpStream::connect((self.imap_server.as_str(), IMAP_PORT))
			.await
			.map_err(ImapError::ConnectionFailed)?;

		// let domain: ServerName<'static> =
		let domain = ServerName::try_from(String::from(&self.imap_server))?;

		tracing::trace!("Establishing a TLS connection");
		let tls_stream = TLS_CONNECTOR
			.connect(domain, tcp_stream)
			.await
			.map_err(ImapError::ConnectionFailed)?;

		Ok(Client::new(tls_stream))
	}

	/// Creates an authenticated session with the IMAP server, passes it to the closure, and automatically logs out when the closure returns.
	///
	/// Passes &mut self as the first parameter to the closure to avoid borrowck errors.
	/// Don't call session.logout() manually in the closure, this function will call it automatically at the end.
	async fn with_session<F, T, E>(&mut self, f: F) -> Result<T, E>
	where
		F: AsyncFnOnce(&mut Email, &mut Session<TlsStream<TcpStream>>) -> Result<T, E>,
		E: From<ImapError>,
	{
		let client = self.client().await?;

		// authenticate and create a session
		let mut session = match &mut self.auth {
			Auth::GmailOAuth2(auth) => {
				authenticate_google_oauth2(client, auth, &self.email).await?
			}
			Auth::Password(password) => {
				authenticate_password(client, &self.email, password).await?
			}
		};

		match f(self, &mut session).await {
			Ok(t) => {
				session.logout().await.map_err(ImapError::Other)?;
				Ok(t)
			}
			Err(e) => {
				// try to log out anyways
				_ = session.logout().await;
				Err(e)
			}
		}
	}
	async fn fetch_impl(&mut self) -> Result<Vec<Entry>, EmailError> {
		self.with_session(async |this, session| {
			tracing::debug!("Fetching emails");

			session.examine("INBOX").await.map_err(ImapError::Other)?;

			let search_string = {
				let mut tmp = "UNSEEN ".to_owned();

				if let Some(sender) = &this.filters.sender {
					_ = write!(tmp, r#"FROM "{sender}" "#);
				}

				if let Some(subjects) = &this.filters.subjects {
					for s in subjects {
						_ = write!(tmp, r#"SUBJECT "{s}" "#);
					}
				}

				if let Some(ex_subjects) = &this.filters.exclude_subjects {
					for exs in ex_subjects {
						_ = write!(tmp, r#"NOT SUBJECT "{exs}" "#);
					}
				}

				tmp.trim_end().to_owned()
			};

			tracing::debug!(
				"Fetching all emails that match the search string: {:?}",
				search_string,
			);
			let mail_ids = session
				.uid_search(&search_string)
				.await
				.map_err(ImapError::Other)?;

			let unread_num = mail_ids.len();
			if unread_num > 0 {
				tracing::info!("Got {unread_num} unread filtered mails");
			} else {
				tracing::debug!(
					"All email for the search query have already been read, none remaining to send"
				);
				return Ok(Vec::new());
			}

			let mail_id_search_str = mail_ids
				.iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>()
				.join(",");

			tracing::trace!("Fetching all email bodies via the UIDs returned from the search");
			let mails = session
				.uid_fetch(&mail_id_search_str, "BODY[]")
				.await
				.map_err(ImapError::Other)?;

			let entries = mails
				.map(|mail| {
					let mail = mail.map_err(ImapError::Other)?;

					let body = mail
						.body()
						.expect("Body should always be present because we explicitly requested it");

					let uid = mail
						.uid
						.expect(
							"UIDs should always be present because we used uid_fetch().\
						The server probably doesn't support them which isn't something ~we~ support for now",
						)
						.into();

					parse(&mailparse::parse_mail(body)?, uid)
				})
				.try_collect::<Vec<Entry>>()
				.await?;

			assert_eq!(mail_ids.len(), entries.len(), "The number of email IDs and the number of fetched email bodies should be the same unless aborted by an error");

			Ok(entries)
		})
		.await
	}

	async fn mark_as_read_impl(&mut self, id: &str) -> Result<(), ImapError> {
		if let ViewMode::ReadOnly = self.view_mode {
			return Ok(());
		}

		self.with_session(async |this, session| {
			session.select("INBOX").await?;

			match this.view_mode {
				ViewMode::MarkAsRead => {
					session
						.uid_store(id, "+FLAGS.SILENT (\\Seen)")
						.await?
						.try_collect::<Vec<_>>()
						.await?;

					tracing::debug!("Marked email uid {id} as read");
				}
				ViewMode::Delete => {
					session
						.uid_store(id, "+FLAGS.SILENT (\\Deleted)")
						.await?
						.try_collect::<Vec<_>>()
						.await?;

					session
						.uid_expunge(id)
						.await?
						.try_collect::<Vec<_>>()
						.await?;
					tracing::debug!("Deleted email uid {id}");
				}
				ViewMode::ReadOnly => unreachable!(),
			}

			Ok(())
		})
		.await
	}
}

fn parse(mail: &ParsedMail, id: EntryId) -> Result<Entry, EmailError> {
	tracing::trace!("Parsing the contents of an email with UID {id:?}");
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

async fn authenticate_google_oauth2(
	mut client: Client<TlsStream<TcpStream>>,
	google_auth: &mut crate::auth::Google,
	email: &str,
) -> Result<Session<TlsStream<TcpStream>>, ImapError> {
	tracing::trace!("Logging into IMAP with Google OAuth2");
	let _greeting = client.read_response().await;
	let session = client
		.authenticate(
			"XOAUTH2",
			google_auth
				.as_imap_oauth2(email)
				.await
				.map_err(ImapError::GoogleOAuth2)?,
		)
		.await;

	match session {
		Ok(session) => {
			tracing::trace!("Authenticated successfully");
			Ok(session)
		}
		Err((e, mut client)) => {
			tracing::error!("Denied access to IMAP via OAuth2: {e}");
			tracing::info!("Refreshing OAuth2 access token and trying again");

			google_auth
				.get_new_access_token()
				.await
				.map_err(ImapError::GoogleOAuth2)?;

			let _greeting = client.read_response().await;

			client
				.authenticate(
					"XOAUTH2",
					google_auth
						.as_imap_oauth2(email)
						.await
						.map_err(ImapError::GoogleOAuth2)?,
				)
				.await
				.map_err(|(e, _)| ImapError::Auth(e))
		}
	}
}

async fn authenticate_password(
	client: Client<TlsStream<TcpStream>>,
	email: &str,
	password: &str,
) -> Result<Session<TlsStream<TcpStream>>, ImapError> {
	tracing::warn!("Logging in to IMAP with a password, this is insecure");

	client
		.login(email, password)
		.await
		.map_err(|(e, _)| ImapError::Auth(e))
}

impl Debug for Email {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Email")
			.field("imap_server", &self.imap_server)
			.field(
				"auth_type",
				match self.auth {
					Auth::Password(_) => &"password",
					Auth::GmailOAuth2(_) => &"gmail_oauth2",
				},
			)
			.field("email", &self.email)
			.field("filters", &self.filters)
			.field("view_mode", &self.view_mode)
			.finish()
	}
}
