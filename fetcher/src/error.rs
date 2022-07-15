/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::path::PathBuf;

use fetcher_core::error::GoogleOAuth2Error;

#[derive(thiserror::Error, Debug)]
pub(crate) enum ConfigError {
	#[error("Twitter API key isn't set up")]
	TwitterApiKeysMissing,

	#[error("Google OAuth2 token isn't set up")]
	GoogleOAuth2TokenMissing,

	#[error("Email password isn't set up")]
	EmailPasswordMissing,

	#[error("Telegram bot token isn't set up")]
	TelegramBotTokenMissing,

	#[error("Error reading config {1}")]
	Read(#[source] std::io::Error, PathBuf),

	#[error("Config {1} is corrupted")]
	CorruptedConfig(
		#[source] Box<(dyn std::error::Error + Send + Sync)>,
		PathBuf,
	),

	#[error("Error writing to config {1}")]
	Write(#[source] std::io::Error, PathBuf),

	#[error("Template {template} not found for task {from_task}")]
	TemplateNotFound { template: String, from_task: String },

	#[error("Xdg error")]
	Xdg(#[from] xdg::BaseDirectoriesError),

	#[error("Error reading stdin")]
	StdinRead(#[source] std::io::Error),

	#[error("Error writing to stdout")]
	StdoutWrite(#[source] std::io::Error),

	#[error("Wrong Google OAuth2 token")]
	GoogleOAuth2WrongToken(#[from] GoogleOAuth2Error),

	#[error("Error setting up an HTTP connection")]
	FetcherCoreHttp(#[from] fetcher_core::error::source::HttpError),

	#[error("Error setting up a read filter")]
	FetcherCoreReadFilter(#[source] Box<fetcher_core::error::source::Error>),
}
