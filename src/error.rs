/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::{error::Error as StdError, io, path::PathBuf};

type BoxError = Box<dyn StdError + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	// disk io stuff
	#[error("XDG error")]
	Xdg(#[from] xdg::BaseDirectoriesError),

	#[error("Inaccessible config file")]
	InaccessibleConfig(io::Error),

	#[error("Inaccessible data file ({1})")]
	InaccessibleData(io::Error, PathBuf),

	#[error("Corrupted data file ({1})")]
	CorruptedData(serde_json::error::Error, PathBuf),

	#[error("Error writing into {1}")]
	Write(io::Error, PathBuf),

	#[error("Invalid config")]
	InvalidConfig(toml::de::Error),

	// stdin & stdout stuff
	#[error("stdin error")]
	Stdin(io::Error),
	#[error("stdout error")]
	Stdout(io::Error),

	// network stuff
	#[error("Network error")]
	Network(BoxError),

	#[error("Google auth: {0}")]
	GoogleAuth(String),

	#[error("Email parse error")]
	EmailParse(#[from] mailparse::MailParseError),

	#[error("IMAP error")]
	Email(imap::Error),

	#[error("Twitter error: {0}")]
	Twitter(egg_mode::error::Error),

	#[error("RSS error")]
	Rss(#[from] rss::Error),

	#[error("Telegram request error")]
	Telegram(#[from] teloxide::RequestError),

	#[error("Invalid DateTime format")]
	InvalidDateTimeFormat(#[from] chrono::format::ParseError),
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
			e => Self::Email(e),
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
