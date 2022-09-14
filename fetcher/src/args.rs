/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use argh::FromArgs;
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
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum TopLvlSubcommand {
	Run(Run),
	Save(Save),
}

/// run all tasks
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "run")]
pub struct Run {
	#[argh(subcommand)]
	pub subcommand: Option<RunSubcommand>,

	/// run once (instead of looping forever)
	#[argh(switch)]
	pub once: bool,

	// TODO: implement dry running
	/// make no permanent changes to the fs or other io
	#[argh(switch)]
	pub dry_run: bool,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum RunSubcommand {
	Task(Task),
}

/// TODO: run a custom task
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "task")]
pub struct Task {}

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
