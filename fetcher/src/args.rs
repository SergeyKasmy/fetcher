/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::named::{JobName, JobWithTaskNames};

use argh::FromArgs;
use color_eyre::Report;
use std::{path::PathBuf, str::FromStr};

/// Automatic news fetching and parsing
#[derive(FromArgs, Debug)]
pub struct Args {
	#[argh(subcommand)]
	pub subcommand: Option<TopLvlSubcommand>,

	/// config path
	#[argh(option)]
	pub config_path: Option<PathBuf>,

	/// data path
	#[argh(option)]
	pub data_path: Option<PathBuf>,

	/// log path
	#[argh(option)]
	pub log_path: Option<PathBuf>,

	/// print version and exit
	#[argh(switch, short = 'v', long = "version")]
	pub print_version: bool,
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

/// Run all jobs. Default if started with no command
#[allow(clippy::struct_excessive_bools)]
#[derive(FromArgs, Default, Debug)]
#[argh(subcommand, name = "run")]
pub struct Run {
	/// run once (instead of looping forever)
	#[argh(switch)]
	pub once: bool,

	/// don't filter out already read entries
	#[argh(switch)]
	pub no_skip_read: bool,

	/// dry run, make no permanent changes to the system
	#[argh(switch)]
	pub dry_run: bool,

	/// run only these jobs and tasks formatted as "job[:task]..."
	#[argh(positional)]
	pub run_filter: Vec<String>,
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
pub struct MarkOldAsRead {
	/// mark only these jobs and tasks as read, formatted as "job[:task]..."
	#[argh(positional)]
	pub run_filter: Vec<String>,
}

/// Load all tasks from the config files and verify their format
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "verify")]
pub struct Verify {
	/// verify only these jobs
	#[argh(positional)]
	pub job_run_filter: Vec<String>,
}

/// Save a setting
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
	Discord,
	Twitter,
}

impl FromStr for Setting {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"google_oauth" => Self::GoogleOAuth2,
			"email_password" => Self::EmailPassword,
			"telegram" => Self::Telegram,
			"discord" => Self::Discord,
			"twitter" => Self::Twitter,
			s => return Err(format!("{s:?} is not a valid setting. Available settings: google_oauth, email_password, telegram, twitter")),
		})
	}
}

/// Wrapper around Job foreign struct to implement `FromStr` from valid job JSON
#[derive(Debug)]
pub struct JsonJob(pub JobName, pub JobWithTaskNames);

impl FromStr for JsonJob {
	type Err = Report;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use fetcher_config::jobs::external_data::ProvideExternalData;
		use fetcher_core::read_filter::ReadFilter;

		struct EmptyExternalData;

		// TODO: add a way to provide external settings even in manual jobs
		impl ProvideExternalData for EmptyExternalData {
			// it's a lie but don't tell anybody...
			type ReadFilter = Box<dyn ReadFilter>;
		}

		let config_job: fetcher_config::jobs::Job = serde_json::from_str(s)?;
		let job = config_job.parse("Manual".to_owned().into(), &EmptyExternalData)?;

		Ok(Self(job.0, job.1))
	}
}
