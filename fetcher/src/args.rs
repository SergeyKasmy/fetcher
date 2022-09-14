/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use argh::FromArgs;

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
}

/// run all tasks
#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
pub struct Run {
	/// run once (instead of looping forever)
	#[argh(switch)]
	pub once: bool,
}
