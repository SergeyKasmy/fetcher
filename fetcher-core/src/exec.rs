/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Exec`] source and sink. It is re-exported in the [`crate::sink`] and [`crate::source`] modules

use async_trait::async_trait;
use std::{io, process::Stdio, string::FromUtf8Error};
use tokio::{io::AsyncWriteExt, process::Command};

use crate::{
	entry::Entry,
	sink::{error::SinkError, Message, MessageId, Sink},
	source::{error::SourceError, Fetch},
};

/// Exec source. It can execute a shell command and source its stdout
#[derive(Debug)]
pub struct Exec {
	/// The command to execute
	pub cmd: String,
}
/// Errors that happened while executing a process
#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum ExecError {
	#[error("Bad command")]
	BadCommand(#[source] io::Error),

	#[error("Command output is not valid UTF-8")]
	BadUtf8(#[from] FromUtf8Error),

	#[error("Can't start the process")]
	CantStart(#[source] io::Error),

	#[error("Can't pass data to the stdin of the process")]
	CantWriteStdin(#[source] io::Error),
}

#[async_trait]
impl Fetch for Exec {
	// TODO: maybe, instead of returining a vec, add a &mut Vec output parameter
	// and maybe also a trait method get_vec() that automatically creates a new vec, fetches into it, and returns it
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		// TODO: add support for windows cmd /C
		tracing::debug!("Spawned a shell with command {:?}", self.cmd);
		let out = Command::new("sh")
			.args(["-c", &self.cmd])
			.output()
			.await
			.map_err(ExecError::BadCommand)?
			.stdout;

		let out = String::from_utf8(out).map_err(ExecError::BadUtf8)?;
		tracing::debug!("Got {out:?} from the command");

		Ok(vec![Entry {
			raw_contents: Some(out),
			..Default::default()
		}])
	}
}

#[async_trait]
impl Sink for Exec {
	/// Passes message's body to the stdin of the process. The tag parameter is ignored
	///
	/// # Errors
	/// * if the process couldn't be started
	/// * if the data couldn't be passed to the stdin pipe of the process
	async fn send(
		&self,
		message: Message,
		_tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError> {
		let Some(body) = message.body else {
			return Ok(None);
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

		Ok(None)
	}
}
