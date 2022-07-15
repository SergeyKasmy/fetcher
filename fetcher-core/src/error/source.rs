pub mod parse;

use parse::Error as ParseError;

use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Error parsing data")]
	Parse(#[from] ParseError),

	#[error("Can't create a source with an empty source list")]
	EmptySourceList,

	#[error("Can't read file {}", .1.to_string_lossy())]
	FileRead(#[source] std::io::Error, PathBuf),

	#[error("HTTP error")]
	Http(#[from] HttpError),

	#[error("Email error")]
	Email(#[from] Box<EmailError>),

	#[error("Twitter error")]
	Twitter(#[from] TwitterError),
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
	#[error("Failed to init TLS")]
	TlsInitFailed(#[source] reqwest::Error),

	#[error("Can't send GET request to {1:?}")]
	Get(#[source] reqwest::Error, String),

	#[error("Reqwest Client wasn't initialized")]
	ClientNotInitialized,
}

#[allow(clippy::large_enum_variant)] // the entire enum is already boxed up above
#[derive(thiserror::Error, Debug)]
pub enum EmailError {
	#[error("IMAP connection error")]
	Imap(#[from] ImapError),

	#[error("Error parsing email")]
	Parse(#[from] mailparse::MailParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum ImapError {
	#[error("Failed to init TLS")]
	TlsInitFailed(#[source] imap::Error),

	#[error("Error authenticating with Google")]
	GoogleAuth(#[source] Box<crate::Error>),

	#[error("Authentication error")]
	Auth(#[source] imap::Error),

	#[error(transparent)]
	Other(#[from] imap::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum TwitterError {
	#[error("Authentication failed")]
	Auth(#[source] egg_mode::error::Error),

	#[error(transparent)]
	Other(#[from] egg_mode::error::Error),
}
