/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Exec`] source

use tokio::process::Command;

use crate::entry::Entry;
use crate::error::source::ExecError;

/// Exec source. It can execute a shell command and source its stdout
#[derive(Debug)]
pub struct Exec {
	/// The command to execute
	pub cmd: String,
}

impl Exec {
	/// Execute the command and returns its stdout in the [`Entry::raw_contents`] field
	#[tracing::instrument(skip_all)]
	pub async fn get(&self) -> Result<Entry, ExecError> {
		let out = Command::new("sh")
			.args(["-c", &self.cmd])
			.output()
			.await?
			.stdout;
		let out = String::from_utf8(out)?;

		Ok(Entry {
			raw_contents: Some(out),
			..Default::default()
		})
	}
}
