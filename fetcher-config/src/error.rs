/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::error::GoogleOAuth2Error;

// TODO: rename to just Error
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
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

	// used with read_filter::get for now
	#[error(transparent)]
	IoError(#[from] std::io::Error),

	#[error("Wrong Google OAuth2 token")]
	GoogleOAuth2WrongToken(#[from] GoogleOAuth2Error),

	#[error("Error setting up HTTP client")]
	FetcherCoreHttp(#[from] fetcher_core::error::source::HttpError),

	#[error("Error setting up HTML parser")]
	FetcherCoreHtml(#[from] fetcher_core::error::transform::HtmlError),

	#[error("Error setting up Regex parser")]
	FetcherCoreRegex(#[from] fetcher_core::error::transform::RegexError),

	#[error("Error setting up a source")]
	FetcherCoreSource(#[source] Box<fetcher_core::error::source::Error>),
}

/*
 * Unused error variants

#[error("The read filter type set in the config is different from the one saved on disk. Read filter type migration is currently unsupported. Either change the read filter type in the config from \"{in_config}\" to \"{on_disk}\", or manually remove the read filter save file at \"{disk_rf_path}\" to create a new one with type \"{in_config}\"")]
IncompatibleReadFilterTypes {
	in_config: fcore::read_filter::Kind,
	on_disk: fcore::read_filter::Kind,
	disk_rf_path: PathBuf,
},
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

#[error("Error reading stdin")]
StdinRead(#[source] std::io::Error),

#[error("Error writing to stdout")]
StdoutWrite(#[source] std::io::Error),
*/
