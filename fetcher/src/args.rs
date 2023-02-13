/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use argh::FromArgs;
use fetcher_core::job::Job;
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
	RunManual(RunManual),
	MarkOldAsRead(MarkOldAsRead),
	Verify(Verify),
	Save(Save),
}

/// run all jobs
#[allow(clippy::struct_excessive_bools)]
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "run")]
pub struct Run {
	/// run once (instead of looping forever)
	#[argh(switch)]
	pub once: bool,

	/// dry run, make no permanent changes to the system
	#[argh(switch)]
	pub dry_run: bool,

	/// run only these jobs
	#[argh(positional)]
	pub job_names: Vec<String>,
}

/// Run a job from the command line formatted as JSON
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "run-manual")]
pub struct RunManual {
	/// run this job, formatted in JSON
	#[argh(positional)]
	pub job: JsonJob,
}

/// Load all tasks from the config files and mark all old entries as read
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "mark-old-as-read")]
pub struct MarkOldAsRead {}

/// Load all tasks from the config files and verify their format
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "verify")]
pub struct Verify {}

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

/// Wrapper around Parsed Task foreign struct to implement `FromStr` from valid task JSON
#[derive(Debug)]
pub struct JsonJob(pub Job);

impl FromStr for JsonJob {
	type Err = serde_json::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use fetcher_config::jobs::external_data::{ExternalDataResult, ProvideExternalData};
		use fetcher_core::read_filter::ReadFilter;

		struct EmptyExternalData;

		impl ProvideExternalData for EmptyExternalData {
			fn twitter_token(&self) -> ExternalDataResult<(String, String)> {
				ExternalDataResult::Unavailable
			}

			fn google_oauth2(&self) -> ExternalDataResult<fetcher_core::auth::Google> {
				ExternalDataResult::Unavailable
			}

			fn email_password(&self) -> ExternalDataResult<String> {
				ExternalDataResult::Unavailable
			}

			fn telegram_bot_token(&self) -> ExternalDataResult<String> {
				ExternalDataResult::Unavailable
			}

			fn read_filter(
				&self,
				_name: &str,
				_expected_rf: fetcher_core::read_filter::Kind,
			) -> ExternalDataResult<ReadFilter> {
				ExternalDataResult::Unavailable
			}
		}

		let config_job: fetcher_config::jobs::Job = serde_json::from_str(s)?;
		Ok(Self(
			config_job.parse("Manual", &EmptyExternalData).unwrap(),
		))
	}
}
