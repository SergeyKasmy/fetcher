/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::settings::{context::StaticContext, external_data_provider::ExternalDataFromDataDir};
use fetcher_config::jobs::{
	Job as JobConfig,
	named::{JobName, JobWithTaskNames},
};

use argh::FromArgs;
use color_eyre::{Report, Result};
use std::{path::PathBuf, str::FromStr};

/// Automation and scalping tool
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

	/// run only these jobs and tasks formatted as "job\[:task\]..."
	#[argh(positional)]
	pub run_filter: Vec<String>,
}

/// Run a job from the command line formatted as JSON
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "run-manual")]
pub struct RunManual {
	/// run this job, formatted in JSON
	#[argh(positional)]
	pub job_config: JsonJobConfig,
}

/// Load all tasks from the config files and mark all old entries as read
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "mark-old-as-read")]
pub struct MarkOldAsRead {
	/// mark only these jobs and tasks as read, formatted as "job\[:task\]..."
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
			s => {
				return Err(format!(
					"{s:?} is not a valid setting. Available settings: google_oauth, email_password, telegram, twitter"
				));
			}
		})
	}
}

/// Wrapper around Job foreign struct to implement `FromStr` from valid job in JSON format
#[derive(Debug)]
pub struct JsonJobConfig(Vec<(JobName, JobConfig)>);

impl FromStr for JsonJobConfig {
	type Err = Report;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// HACK: if the input is a JSON array
		let config_jobs = if s.chars().next().expect("Manual job JSON is empty") == '[' {
			let v: Vec<fetcher_config::jobs::Job> = serde_json::from_str(s)?;
			v.into_iter()
				.enumerate()
				// use the index as the job name
				.map(|(idx, job)| ((idx + 1).to_string().into(), job))
				.collect()
		} else {
			let job: fetcher_config::jobs::Job = serde_json::from_str(s)?;
			// just use "Manual" as the job name
			vec![("Manual".to_owned().into(), job)]
		};

		Ok(Self(config_jobs))
	}
}

impl JsonJobConfig {
	pub fn decode(
		self,
		cx: StaticContext,
	) -> Result<impl Iterator<Item = (JobName, JobWithTaskNames)>> {
		Ok(self
			.0
			.into_iter()
			.map(|(name, config)| config.decode_from_conf(name, &ExternalDataFromDataDir { cx }))
			.collect::<Result<Vec<_>, _>>()?
			.into_iter())
	}
}
