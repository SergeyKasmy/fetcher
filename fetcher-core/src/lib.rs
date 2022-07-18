/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
#![warn(clippy::pedantic)]
// #![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]

pub mod auth;
pub mod entry;
pub mod error;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;

use crate::entry::Entry;
use crate::error::Error;
use crate::error::ErrorChainExt;
use crate::sink::Sink;
use crate::source::Source;
use crate::task::Task;

/// Run a task (both the source and the sink part) once to completion
///
/// # Errors
/// If there was an error fetching the data, sending the data, or saving what data was successfully sent to an external location
pub async fn run_task(t: &mut Task) -> Result<(), Error> {
	tracing::trace!("Running task: {:#?}", t);

	let fetch = async {
		for entry in t.source.get(t.parsers.as_deref()).await?.into_iter().rev() {
			process_entry(&mut t.sink, entry, t.tag.as_deref(), &mut t.source).await?;
		}

		Ok::<(), Error>(())
	};

	match fetch.await {
		Ok(_) => (),
		Err(e) => {
			if let Some(network_err) = e.is_connection_error() {
				tracing::warn!("Network error: {}", network_err.display_chain());
			} else {
				return Err(e);
			}
		}
	}

	Ok(())
}

/// Send an entry and mark it as read afterwards
///
/// # Errors
/// If there was an error sending the entry or marking it as read
#[tracing::instrument(name = "entry", skip_all, fields(id = entry.id))]
async fn process_entry(
	sink: &mut Sink,
	entry: Entry,
	tag: Option<&str>,
	mark_as_read: &mut Source,
) -> Result<(), Error> {
	tracing::trace!("Processing entry: {entry:?}");

	sink.send(entry.msg, tag).await?;

	if let Some(id) = &entry.id {
		mark_as_read.mark_as_read(id).await?;
	}

	Ok::<(), Error>(())
}
