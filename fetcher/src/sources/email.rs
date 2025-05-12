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
	error::FetcherError,
	sinks::message::Message,
	sources::error::SourceError,
};

use async_imap::Client;
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
	rustls::{ClientConfig, RootCertStore, crypto::aws_lc_rs, pki_types::ServerName},
};

const IMAP_PORT: u16 = 993;

static TLS_CONNECTOR: LazyLock<TlsConnector> = LazyLock::new(|| {
	// FIXME: rustls docs say default process-wide providers should never be set in libraries
	// https://docs.rs/rustls/0.23.22/rustls/crypto/struct.CryptoProvider.html
	// I guess we should try to get the default provider and use aws_lc otherwise jusr for this ClientConfig
	aws_lc_rs::default_provider().install_default().unwrap();

	let mut root_cert_store = RootCertStore::empty();
	root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

	let config = ClientConfig::builder()
		.with_root_certificates(root_cert_store)
		.with_no_client_auth();

	let connector = TlsConnector::from(Arc::new(config));

	connector
});

// FIXME: blocks the runtime. Probably migrate to imap-async crate or wrap in spawn_blocking
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

	#[error(transparent)]
	GoogleOAuth2(#[from] GoogleAuthError),

	#[error("Authentication error")]
	Auth(#[source] async_imap::error::Error),

	#[error(transparent)]
	Other(#[from] async_imap::error::Error),
}

// I'd make that a function but the imap crate didn't want to agree with me
macro_rules! authenticate {
	($login:expr, $auth:expr, $client:expr) => {{
		let auth = $auth;

		match auth {
			Auth::GmailOAuth2(auth) => {
				tracing::trace!("Logging into IMAP with Google OAuth2");

				// FIXME: don't crash
				let _greeting = $client
					.read_response()
					.await
					.expect("unexpected end of stream, expected greeting")
					.expect("unexpected error, expected greeting");

				let session = $client
					.authenticate(
						"XOAUTH2",
						auth.as_imap_oauth2($login)
							.await
							.map_err(ImapError::GoogleOAuth2)?,
					)
					.await;

				match session {
					Ok(session) => {
						tracing::trace!("Authenticated successfully");
						session
					}
					// refresh access token and retry
					Err((e, mut client)) => {
						tracing::error!("Denied access to IMAP via OAuth2: {e}");
						tracing::info!("Refreshing OAuth2 access token and trying again");

						auth.get_new_access_token()
							.await
							.map_err(ImapError::GoogleOAuth2)?;

						// FIXME: don't crash
						let _greeting = client
							.read_response()
							.await
							.expect("unexpected end of stream, expected greeting")
							.expect("unexpected error, expected greeting");

						client
							.authenticate(
								"XOAUTH2",
								auth.as_imap_oauth2($login)
									.await
									.map_err(ImapError::GoogleOAuth2)?,
							)
							.await
							.map_err(|(e, _)| ImapError::Auth(e))?
					}
				}
			}
			Auth::Password(password) => {
				tracing::warn!("Logging in to IMAP with a password, this is insecure");

				$client
					.login($login, password)
					.await
					.map_err(|(e, _)| ImapError::Auth(e))?
			}
		}
	}};
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
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		self.fetch_impl().await.map_err(Into::into)
	}
}

impl MarkAsRead for Email {
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError> {
		self.mark_as_read_impl(id)
			.await
			.map_err(|e| FetcherError::from(SourceError::from(EmailError::from(e))))
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

		let domain: ServerName<'static> =
			ServerName::try_from(String::from(&self.imap_server)).unwrap();

		tracing::trace!("Establishing a TLS connection");
		let tls_stream = TLS_CONNECTOR.connect(domain, tcp_stream).await.unwrap();

		Ok(Client::new(tls_stream))
	}

	async fn fetch_impl(&mut self) -> Result<Vec<Entry>, EmailError> {
		tracing::debug!("Fetching emails");

		let mut client = self.client().await?;
		let mut session = authenticate!(&self.email, &mut self.auth, client);

		session.examine("INBOX").await.map_err(ImapError::Other)?;

		let search_string = {
			let mut tmp = "UNSEEN ".to_owned();

			if let Some(sender) = &self.filters.sender {
				_ = write!(tmp, r#"FROM "{sender}" "#);
			}

			if let Some(subjects) = &self.filters.subjects {
				for s in subjects {
					_ = write!(tmp, r#"SUBJECT "{s}" "#);
				}
			}

			if let Some(ex_subjects) = &self.filters.exclude_subjects {
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
			return Ok(Vec::new());
		}

		tracing::trace!("Fetching all email bodies via the UIDs returned from the search");
		let mails = session
			.uid_fetch(&mail_ids, "BODY[]")
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
					.to_string();

				parse(&mailparse::parse_mail(body)?, uid)
			})
			.try_collect::<Vec<Entry>>()
			.await?;

		// FIXME: doesn't logout if early returned with an error. I think it should...
		session.logout().await.map_err(ImapError::Other)?;

		Ok(entries)
	}

	async fn mark_as_read_impl(&mut self, id: &str) -> Result<(), ImapError> {
		if let ViewMode::ReadOnly = self.view_mode {
			return Ok(());
		}

		let mut client = self.client().await?;
		let mut session = authenticate!(&self.email, &mut self.auth, client);

		session.select("INBOX").await?;

		match self.view_mode {
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
		};

		session.logout().await?;

		Ok(())
	}
}

fn parse(mail: &ParsedMail, id: String) -> Result<Entry, EmailError> {
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
		id: Some(id.into()),
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
