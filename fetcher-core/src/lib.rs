/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! fetcher core    // TODO
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)] // TODO
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]

pub mod action;
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

use crate::{
	action::Action,
	entry::Entry,
	error::{transform::Error as TransformError, Error},
	task::Task,
};

use std::collections::HashSet;

/// Run a task (both the source and the sink part) once to completion
///
/// # Errors
/// If there was an error fetching the data, sending the data, or saving what data was successfully sent to an external location
pub async fn run_task(t: &mut Task) -> Result<(), Error> {
	tracing::trace!("Running task: {:#?}", t);

	let entries = {
		let raw = t.source.get().await?;

		let processed = match &t.actions {
			Some(actions) => process_entries(raw, actions).await?,
			None => raw,
		};

		// TODO: make this an action mb?
		remove_duplicates(processed)
	};

	// entries should be sorted newest to oldest but we should send oldest first
	for entry in entries.into_iter().rev() {
		t.sink.send(entry.msg, t.tag.as_deref()).await?;

		if let Some(id) = &entry.id {
			match &mut t.source {
				source::Source::WithSharedReadFilter(_) => {
					if let Some(rf) = &t.rf {
						rf.write().await.mark_as_read(id).await?;
					}
				}
				source::Source::WithCustomReadFilter(x) => x.mark_as_read(id).await?,
			}
		}
	}

	Ok(())
}

async fn process_entries(
	mut entries: Vec<Entry>,
	actions: &[Action],
) -> Result<Vec<Entry>, TransformError> {
	for a in actions {
		entries = a.process(entries).await?;
	}

	Ok(entries)
}

fn remove_duplicates(entries: Vec<Entry>) -> Vec<Entry> {
	let mut uniq = Vec::new();
	let mut used_ids = HashSet::new();

	for ent in entries {
		match ent.id.as_deref() {
			Some("") => panic!("An id should never be none but empty"),
			Some(id) => {
				if used_ids.insert(id.to_owned()) {
					uniq.push(ent);
				}
			}
			None => uniq.push(ent),
		}
	}

	uniq
}
