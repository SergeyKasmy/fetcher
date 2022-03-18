/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: create a type that wraps the Error enum with the name of the task at the task level

use std::{error::Error as StdError, io, path::PathBuf};

type BoxError = Box<dyn StdError + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	// disk io stuff
	#[error("XDG error")]
	Xdg(#[from] xdg::BaseDirectoriesError),

	#[error("Inaccessible config file ({1})")]
	InaccessibleConfig(#[source] io::Error, PathBuf),

	#[error("Inaccessible data file ({1})")]
	InaccessibleData(#[source] io::Error, PathBuf),

	#[error("Corrupted data file ({1})")]
	CorruptedData(#[source] serde_json::error::Error, PathBuf),

	#[error("Error writing into {1}")]
	Write(#[source] io::Error, PathBuf),

	#[error("Invalid config {1}")]
	InvalidConfig(#[source] figment::error::Error, PathBuf),

	#[error("Incompatible config values in {1}: {0}")]
	IncompatibleConfigValues(&'static str, PathBuf),

	#[error("Template {0} not found")]
	TemplateNotFound(PathBuf),

	// stdin & stdout stuff
	#[error("stdin error")]
	Stdin(#[source] io::Error),
	#[error("stdout error")]
	Stdout(#[source] io::Error),

	// network stuff
	#[error("Network error")]
	Network(#[source] BoxError),

	#[error("Google auth error: {0}")]
	GoogleAuth(String),

	#[error("Email parse error")]
	EmailParse(#[from] mailparse::MailParseError),

	#[error("IMAP error")]
	Email(#[source] Box<imap::Error>), // box to avoid big uneven enum size

	#[error("Twitter error")]
	Twitter(#[source] egg_mode::error::Error),

	#[error("RSS error")]
	Rss(#[from] rss::Error),

	#[error("HTML error: {0}")]
	Html(&'static str), // TODO: add more context

	#[error("Telegram request error\nMessage: {1:?}")]
	Telegram(
		#[source] teloxide::RequestError,
		Box<dyn std::fmt::Debug + Send + Sync>,
	),

	#[error("Invalid DateTime format")]
	InvalidDateTimeFormat(#[from] chrono::format::ParseError),

	#[error("Other err: {0}")]
	Other(String),
}

impl From<reqwest::Error> for Error {
	fn from(e: reqwest::Error) -> Self {
		Self::Network(Box::new(e))
	}
}

impl From<imap::Error> for Error {
	fn from(e: imap::Error) -> Self {
		match e {
			imap::Error::Io(io_err) => Error::Network(Box::new(io_err)),
			e => Self::Email(Box::new(e)),
		}
	}
}

impl From<egg_mode::error::Error> for Error {
	fn from(e: egg_mode::error::Error) -> Self {
		match e {
			egg_mode::error::Error::NetError(e) => Self::Network(Box::new(e)),
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
