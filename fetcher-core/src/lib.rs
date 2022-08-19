/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! fetcher core    // TODO
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// #![warn(missing_docs)]	// FIXME
// #![warn(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

/// Everything concerning some kind of non-primitive authentication
pub mod auth;
/// Contains [`Entry`] - a struct that contains a message that can be fed into a [`Sink`] and an id that can be used with a [`ReadFilter`](`read_filter::ReadFilter`)
pub mod entry;
/// Every error this crate and any of its modules may return, plus some helper functions
pub mod error;
/// Filtering already read entries and marking what entries have already been read
pub mod read_filter;
/// Sending fetched data
pub mod sink;
/// Fetching data
pub mod source;
/// Contains [`Task`] - a struct that combines everything from the above into a one coherent entity
pub mod task;
pub mod transform;

use crate::entry::Entry;
use crate::error::Error;
use crate::sink::Sink;
use crate::source::Source;
use crate::task::Task;

/// Run a task (both the source and the sink part) once to completion
///
/// # Errors
/// If there was an error fetching the data, sending the data, or saving what data was successfully sent to an external location
pub async fn run_task(t: &mut Task) -> Result<(), Error> {
	tracing::trace!("Running task: {:#?}", t);

	for entry in t
		.source
		.get(t.transforms.as_deref())
		.await?
		.into_iter()
		.rev()
	{
		process_entry(&mut t.sink, entry, t.tag.as_deref(), &mut t.source).await?;
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
	tracing::trace!("Processing entry: {entry:#?}");

	sink.send(entry.msg, tag).await?;

	if let Some(id) = &entry.id {
		mark_as_read.mark_as_read(id).await?;
	}

	Ok::<(), Error>(())
}
