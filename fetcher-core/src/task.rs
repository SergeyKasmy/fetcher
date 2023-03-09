/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic block of [`fetcher`](`crate`) that is a [`Task`]

pub mod entry_to_msg_map;

use std::collections::HashSet;

use self::entry_to_msg_map::EntryToMsgMap;
use crate::{
	action::{transform::error::TransformError, Action},
	entry::Entry,
	error::Error,
	sink::Sink,
	source::Source,
};

/// A core primitive of [`fetcher`](`crate`).
/// Contains everything from a [`Source`] that allows to fetch some data, to a [`Sink`] that takes that data and sends it somewhere.
/// It also contains any transformators
#[derive(Debug)]
pub struct Task {
	/// An optional name/tag that may be put near a message body to differentiate this task from others that may be similar
	pub name: Option<String>,

	/// The source where to fetch some data from
	pub source: Option<Box<dyn Source>>,

	/// A list of optional transformators which to run the data received from the source through
	pub actions: Option<Vec<Action>>,

	/// The sink where to send the data to
	pub sink: Option<Box<dyn Sink>>,

	/// Map of an entry to a message. Used when an entry is a reply to an older entry to be able to show that as a message, too
	pub entry_to_msg_map: Option<EntryToMsgMap>,
}

impl Task {
	/// Run a task (both the source and the sink part) once to completion
	///
	/// # Errors
	/// If there was an error fetching the data, sending the data, or saving what data was successfully sent to an external location
	#[tracing::instrument(fields(name = self.name), skip(self))]
	pub async fn run(&mut self) -> Result<(), Error> {
		tracing::trace!("Running task");

		let entries = {
			let raw = match &mut self.source {
				Some(source) => source.fetch().await?,
				None => vec![Entry::default()], // return just an empty entry if there is no source
			};

			tracing::debug!("Got {} raw entries from the source(s)", raw.len());

			let processed = match &self.actions {
				Some(actions) => process_entries(raw, actions).await?,
				None => raw,
			};

			tracing::debug!("Got {} fully processed entries", processed.len());

			remove_duplicates(processed)
		};

		// entries should be sorted newest to oldest but we should send oldest first
		for entry in entries.into_iter().rev() {
			let msgid = match self.sink.as_ref() {
				Some(sink) => {
					// use raw_contents as msg.body if the message is empty
					let mut msg = entry.msg;
					if msg.title.is_none()
						&& msg.body.is_none() && msg.link.is_none()
						&& msg.media.is_none()
					{
						msg.body = entry.raw_contents.clone();
					}

					let tag = self.name.as_deref();

					tracing::debug!("Sending {msg:?} to a sink with tag {tag:?}");
					sink.send(
						msg,
						dbg!(self.entry_to_msg_map.as_mut())
							.and_then(|map| map.get_if_exists(entry.id.as_ref())),
						self.name.as_deref(),
					)
					.await?
				}
				None => None,
			};

			if let Some(entry_id) = entry.id {
				if let Some(source) = &mut self.source {
					tracing::debug!("Marking {entry_id:?} as read");
					source.mark_as_read(&entry_id).await?;
				}

				if let Some((msgid, map)) = msgid.zip(self.entry_to_msg_map.as_mut()) {
					tracing::debug!("Associating entry {entry_id:?} with message {msgid:?}");
					map.insert(entry_id, msgid).await?;
				}
			}
		}

		Ok(())
	}
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
	let num_og_entries = entries.len();

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

	let num_removed = num_og_entries - uniq.len();
	if num_removed > 0 {
		tracing::trace!("Removed {} duplicate entries", num_removed);
	}

	uniq
}
