/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use argh::FromArgs;
use std::str::FromStr;

/// fetcher
#[derive(FromArgs)]
pub struct Args {
	#[argh(subcommand)]
	pub inner: Subcommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Subcommands {
	Run(Run),
	Save(Save),
}

/// run all tasks
#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
pub struct Run {
	/// run once (instead of looping forever)
	#[argh(switch)]
	pub once: bool,
}

/// save a setting
#[derive(FromArgs)]
#[argh(subcommand, name = "save")]
pub struct Save {
	/// which setting to save
	#[argh(positional)]
	pub setting: Setting,
}

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
