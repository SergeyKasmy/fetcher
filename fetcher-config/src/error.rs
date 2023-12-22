/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub type Result<T, E = FetcherConfigError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum FetcherConfigError {
	#[error(transparent)]
	ExternalError(#[from] crate::jobs::external_data::ExternalDataError),

	#[error("Twitter API key isn't set up")]
	TwitterApiKeysMissing,

	#[error("Google OAuth2 token isn't set up")]
	GoogleOAuth2TokenMissing,

	#[error("Email password isn't set up")]
	EmailPasswordMissing,

	#[error("Email imap field is missing and it's not clear what it should be")]
	EmailImapFieldMissing,

	#[error("Telegram bot token isn't set up")]
	TelegramBotTokenMissing,

	#[error("Discord bot token isn't set up")]
	DiscordBotTokenMissing,

	#[error("Importing is unavailable")]
	ImportingUnavailable,

	#[error("Wrong Google OAuth2 token")]
	GoogleOAuth2WrongToken(#[from] fetcher_core::auth::google::GoogleOAuth2Error),

	#[error("refresh - every is not a valid duration format, e.g. 1m, 10h, 1d")]
	BadDurationFormat(#[from] duration_str::DError),

	#[error("refresh - at is not a valid time format, e.g. 14:30")]
	BadTimeFormat(#[from] chrono::ParseError),

	#[error("Error setting up HTTP client")]
	FetcherCoreHttp(#[from] fetcher_core::source::http::HttpError),

	#[error("Error setting up HTML parser")]
	FetcherCoreHtml(#[from] fetcher_core::action::transform::entry::html::HtmlError),

	#[error("Error setting up regex")]
	FetcherCoreBadRegex(#[from] fetcher_core::error::BadRegexError),

	#[error("Error setting up extract action")]
	FetcherCoreExtract(#[from] fetcher_core::action::transform::field::extract::ExtractError),

	#[error("Error setting up a source")]
	FetcherCoreSource(#[source] Box<fetcher_core::source::error::SourceError>),
}
