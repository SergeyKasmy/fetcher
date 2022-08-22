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

use itertools::Itertools;

use crate::{
	entry::Entry,
	error::{transform::Error as TransformError, Error},
	task::Task,
	transform::Transform,
};

/// Run a task (both the source and the sink part) once to completion
///
/// # Errors
/// If there was an error fetching the data, sending the data, or saving what data was successfully sent to an external location
pub async fn run_task(t: &mut Task) -> Result<(), Error> {
	tracing::trace!("Running task: {:#?}", t);

	let entries = {
		let untransformed = t.source.get().await?;
		let transformed = if let Some(transforms) = t.transforms.as_deref() {
			transform_entries(untransformed, transforms).await?
		} else {
			untransformed
		};

		transformed
			.into_iter()
			// TODO: I don't like this clone...
			// FIXME: removes all entries with no/empty id because "" == "". Maybe move to .remove_read()?
			.unique_by(|ent| ent.id.clone())
			.collect::<Vec<_>>()
	};

	for entry in entries {
		t.sink.send(entry.msg, t.tag.as_deref()).await?;

		if let Some(id) = &entry.id {
			if let Some(rf) = &t.rf {
				rf.write().await.mark_as_read(id).await?;
			}
		}
	}

	Ok(())
}

async fn transform_entries(
	untransformed: Vec<Entry>,
	transforms: &[Transform],
) -> Result<Vec<Entry>, TransformError> {
	let mut fully_transformed = Vec::new();
	for entry in untransformed {
		let mut to_transform = vec![entry];

		for transform in transforms {
			let mut partially_transformed = Vec::new(); // transformed only with the current transformator

			for entry_to_transform in to_transform {
				partially_transformed.extend(transform.transform(entry_to_transform).await?);
			}

			to_transform = partially_transformed;
		}

		fully_transformed.extend(to_transform);
	}

	Ok(fully_transformed)
}
