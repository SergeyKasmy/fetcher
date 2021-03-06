/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)] // TODO
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

// TODO: more tests

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
