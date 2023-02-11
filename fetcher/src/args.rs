/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use argh::FromArgs;
use fetcher_config::tasks::ParsedTask;
use std::{path::PathBuf, str::FromStr};

/// fetcher
#[derive(FromArgs, Debug)]
pub struct Args {
	#[argh(subcommand)]
	pub subcommand: TopLvlSubcommand,

	/// config path
	#[argh(option)]
	pub config_path: Option<PathBuf>,

	/// data path
	#[argh(option)]
	pub data_path: Option<PathBuf>,

	/// log path
	#[argh(option)]
	pub log_path: Option<PathBuf>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum TopLvlSubcommand {
	Run(Run),
	Save(Save),
}

// TODO: construct a temporary custom task right in the command line
// TODO: maybe remake run modes from bool to enum
/// run all tasks
#[allow(clippy::struct_excessive_bools)]
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "run")]
pub struct Run {
	/// verify only, don't run
	#[argh(switch)]
	pub verify_only: bool,

	/// run once (instead of looping forever)
	#[argh(switch)]
	pub once: bool,

	/// dry run, make no permanent changes to the system
	#[argh(switch)]
	pub dry_run: bool,

	/// mark all old entries from that source as read, implies --once
	#[argh(switch)]
	pub mark_old_as_read: bool,

	/// run this task instead of those saved as config files
	#[argh(option)]
	pub manual: Option<JsonTask>,

	/// run only these tasks
	#[argh(positional)]
	pub tasks: Vec<String>,
}

/// save a setting
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "save")]
pub struct Save {
	/// which setting to save
	#[argh(positional)]
	pub setting: Setting,
}

#[derive(Debug)]
pub enum Setting {
	GoogleOAuth2,
	EmailPassword,
	Telegram,
	Twitter,
}

impl FromStr for Setting {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"google_oauth" => Self::GoogleOAuth2,
			"email_password" => Self::EmailPassword,
			"telegram" => Self::Telegram,
			"twitter" => Self::Twitter,
			s => return Err(format!("{s:?} is not a valid setting. Available settings: google_oauth, email_password, telegram, twitter")),
		})
	}
}

#[derive(Debug)]
pub struct JsonTask(pub ParsedTask);

impl FromStr for JsonTask {
	type Err = serde_json::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use fetcher_config::tasks::external_data::ExternalData;
		use fetcher_core::read_filter::ReadFilter;

		struct EmptyExternalData;

		impl ExternalData for EmptyExternalData {
			fn twitter_token(
				&self,
			) -> fetcher_config::tasks::external_data::ExternalDataResult<Option<(String, String)>>
			{
				Ok(None)
			}

			fn google_oauth2(
				&self,
			) -> fetcher_config::tasks::external_data::ExternalDataResult<
				Option<fetcher_core::auth::Google>,
			> {
				Ok(None)
			}

			fn email_password(
				&self,
			) -> fetcher_config::tasks::external_data::ExternalDataResult<Option<String>> {
				Ok(None)
			}

			fn telegram_bot_token(
				&self,
			) -> fetcher_config::tasks::external_data::ExternalDataResult<Option<String>> {
				Ok(None)
			}

			fn read_filter(
				&self,
				_name: &str,
				_expected_rf: fetcher_core::read_filter::Kind,
			) -> fetcher_config::tasks::external_data::ExternalDataResult<Option<ReadFilter>> {
				Ok(None)
			}
		}

		let config_task: fetcher_config::tasks::Task = serde_json::from_str(s)?;
		Ok(Self(config_task.parse("", &EmptyExternalData).unwrap()))
	}
}
