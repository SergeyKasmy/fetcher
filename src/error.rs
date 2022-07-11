/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: create a type that wraps the Error enum with the name of the task at the task level

use std::{error::Error as StdError, fmt, io, path::PathBuf};

// pub mod parser;

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Error reading {1}")]
	LocalIoRead(#[source] io::Error, PathBuf),

	#[error(
		"Bad path {0}. It probably contains bad/non-unicode characters or just isn't compatible"
	)]
	BadPath(PathBuf),

	// TODO: should these io errors even be in the library crate if config parsing is done in the binary?
	#[error("Error writing into {1}")]
	LocalIoWrite(#[source] io::Error, PathBuf),

	#[error("Error saving read filter data")]
	LocalIoWriteReadFilterData(#[source] io::Error),

	#[error("XDG error")]
	Xdg(#[from] xdg::BaseDirectoriesError),

	#[error("File {1} is corrupted")]
	CorruptedFile(#[source] serde_json::error::Error, PathBuf),

	#[error("Config {1} has invalid format")]
	InvalidConfigFormat(#[source] figment::error::Error, PathBuf),

	#[error("Incompatible config values in {1}: {0}")]
	IncompatibleConfigValues(&'static str, PathBuf),

	#[error("Template {0} not found (from {1})")]
	TemplateNotFound(String, PathBuf),

	#[error("{0} hasn't been set up and thus can't be used")]
	ServiceNotReady(String), // user didn't setup auth/other data for a service before using it

	#[error("stdin error")]
	Stdin(#[source] io::Error),

	#[error("stdout error")]
	Stdout(#[source] io::Error),

	#[error("No internet connection")]
	NoConnection(#[source] BoxError),

	#[error("Google auth error: {0}")]
	GoogleAuth(String),

	#[error("IMAP error")]
	Email(#[source] Box<imap::Error>), // box to avoid big uneven enum size

	#[error("Twitter error")]
	Twitter(#[source] egg_mode::error::Error),

	#[error("RSS error")]
	Rss(#[from] rss::Error),

	#[error("Error parsing HTML: {0} ({1:?})")]
	HtmlParse(
		&'static str,
		/* additional info */ Option<Box<dyn fmt::Debug + Send + Sync>>,
	),

	#[error("Telegram error\nMessage: {1:?}")]
	Telegram(
		#[source] teloxide::RequestError,
		Box<dyn fmt::Debug + Send + Sync>,
	),

	#[error("Error parsing email")]
	EmailParse(#[from] mailparse::MailParseError),

	#[error("Invalid JSON")]
	JsonParseInvalid(#[from] serde_json::error::Error),

	#[error("JSON key {0} not found")]
	JsonParseKeyNotFound(String),

	#[error("JSON key {key} wrong format: expected {expected_type}, found {found_type}")]
	JsonParseKeyWrongType {
		key: String,
		expected_type: &'static str,
		found_type: String,
	},

	#[error("Invalid URL ({1:?})")]
	UrlInvalid(#[source] url::ParseError, String),

	#[error("Invalid DateTime format")]
	InvalidDateTimeFormat(#[from] chrono::format::ParseError),

	#[error("Other error: {0}")]
	Other(String),
}

impl From<reqwest::Error> for Error {
	fn from(e: reqwest::Error) -> Self {
		Self::NoConnection(Box::new(e))
	}
}

impl From<imap::Error> for Error {
	fn from(e: imap::Error) -> Self {
		match e {
			imap::Error::Io(io_err) => Error::NoConnection(Box::new(io_err)),
			e => Self::Email(Box::new(e)),
		}
	}
}

impl From<egg_mode::error::Error> for Error {
	fn from(e: egg_mode::error::Error) -> Self {
		match e {
			egg_mode::error::Error::NetError(e) => Self::NoConnection(Box::new(e)),
			e => Self::Twitter(e),
		}
	}
}

impl
	From<(
		teloxide::RequestError,
		Box<dyn std::fmt::Debug + Send + Sync>,
	)> for Error
{
	fn from(
		(e, msg): (
			teloxide::RequestError,
			Box<dyn std::fmt::Debug + Send + Sync>,
		),
	) -> Self {
		match e {
			teloxide::RequestError::Network(net_err) => net_err.into(),
			other_err => Error::Telegram(other_err, msg),
		}
	}
}
