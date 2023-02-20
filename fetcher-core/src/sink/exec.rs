/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: maybe merge that with the exec source
//! This module contains [`Exec`] sink

use std::process::Stdio;
use tokio::{io::AsyncWriteExt, process::Command};

use super::Message;
use crate::error::sink::ExecError;

/// Exec sink. It can execute a shell command and pass the message body as an argument
#[derive(Debug)]
pub struct Exec {
	/// The command to execute
	pub cmd: String,
}

impl Exec {
	/// Passes message's body to the stdin of the process
	///
	/// # Errors
	/// * if the process couldn't be started
	/// * if the data couldn't be passed to the stdin pipe of the process
	pub async fn send(&self, message: Message) -> Result<(), ExecError> {
		let Some(body) = message.body else {
			return Ok(());
		};

		tracing::debug!("Spawned process {:?}", self.cmd);
		let mut shell = Command::new("sh")
			.arg("-c")
			.arg(&self.cmd)
			.stdin(Stdio::piped())
			.stdout(Stdio::null())
			.spawn()
			.map_err(ExecError::CantStart)?;

		if let Some(stdin) = &mut shell.stdin {
			tracing::debug!("Writing {body:?} to stdin of the process");
			stdin
				.write_all(body.as_bytes())
				.await
				.map_err(ExecError::CantWriteStdin)?;
		}

		tracing::trace!("Waiting for the process to exit");
		shell.wait().await.map_err(ExecError::CantStart)?;
		tracing::trace!("Process successfully exited");

		Ok(())
	}
}
