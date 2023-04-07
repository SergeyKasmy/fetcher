/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic block of [`fetcher`](`crate`) that is a [`Task`]

pub mod entry_to_msg_map;

use self::entry_to_msg_map::EntryToMsgMap;
use crate::{
	action::Action,
	entry::{Entry, EntryId},
	error::Error,
	sink::{
		message::{Message, MessageId},
		Sink,
	},
	source::Source,
};

use std::{borrow::Cow, collections::HashSet};

/// A core primitive of [`fetcher`](`crate`).
/// Contains everything from a [`Source`] that allows to fetch some data, to a [`Sink`] that takes that data and sends it somewhere.
/// It also contains any transformators
#[derive(Debug)]
pub struct Task {
	/// An optional tag that may be put near a message body to differentiate this task from others that may be similar
	pub tag: Option<String>,

	/// The source where to fetch some data from
	pub source: Option<Box<dyn Source>>,

	/// A list of optional transformators which to run the data received from the source through
	pub actions: Option<Vec<Action>>,

	/// Map of an entry to a message. Used when an entry is a reply to an older entry to be able to show that as a message, too
	pub entry_to_msg_map: Option<EntryToMsgMap>,
}

impl Task {
	/// Run a task (both the source and the sink part) once to completion
	///
	/// # Errors
	/// If there was an error fetching the data, sending the data, or saving what data was successfully sent to an external location
	#[tracing::instrument(skip(self))]
	pub async fn run(&mut self) -> Result<(), Error> {
		tracing::trace!("Running task");

		let raw = match &mut self.source {
			Some(source) => source.fetch().await?,
			None => vec![Entry::default()], // return just an empty entry if there is no source
		};

		tracing::debug!("Got {} raw entries from the sources", raw.len());
		tracing::trace!("Raw entries: {raw:#?}");

		self.process_entries(raw).await?;

		Ok(())
	}

	// TODO: figure out a way to split into several functions to avoid 15 level nesting?
	// It's a bit difficult because this function can't be a method because we are borrowing self.actions
	// throughout the entire process
	async fn process_entries(&mut self, mut entries: Vec<Entry>) -> Result<(), Error> {
		for act in self.actions.iter().flatten() {
			match act {
				Action::Filter(f) => {
					f.filter(&mut entries).await;
				}
				Action::Transform(tr) => {
					let mut fully_transformed = Vec::new();

					for entry in entries {
						fully_transformed.extend(tr.transform(entry).await?);
					}

					entries = fully_transformed;
				}
				Action::Sink(s) => {
					let undeduped_len = entries.len();
					tracing::trace!("Entries to send before dedup: {undeduped_len}");

					entries = remove_duplicates(entries);

					if undeduped_len - entries.len() > 0 {
						tracing::info!(
							"Removed {} duplicate entries before sending",
							undeduped_len - entries.len()
						);
					}

					tracing::trace!("Sending entries: {entries:#?}");

					// entries should be sorted newest to oldest but we should send oldest first
					for entry in entries.iter().rev() {
						let msg_id = send_entry(
							&**s,
							self.entry_to_msg_map.as_mut(),
							self.tag.as_deref(),
							entry,
						)
						.await?;

						if let Some(entry_id) = entry.id.as_ref() {
							mark_entry_as_read(
								entry_id,
								msg_id,
								self.source.as_mut(),
								self.entry_to_msg_map.as_mut(),
							)
							.await?;
						}
					}
				}
			}
		}

		Ok(())
	}
}

#[tracing::instrument(level = "trace", skip_all, fields(entry_id = ?entry.id))]
async fn send_entry(
	sink: &dyn Sink,
	mut entry_to_msg_map: Option<&mut EntryToMsgMap>,
	tag: Option<&str>,
	entry: &Entry,
) -> Result<Option<MessageId>, Error> {
	tracing::trace!("Sending entry");

	// send message if it isn't empty or raw_contents of they aren't
	if entry.msg.is_empty() && entry.raw_contents.is_none() {
		return Ok(None);
	}

	let msg = if entry.msg.is_empty() {
		Cow::Owned(Message {
			body: Some(
				entry
					.raw_contents
					.clone()
					.expect("raw_contents should be some because of the early return check"),
			),
			..entry.msg.clone()
		})
	} else {
		Cow::Borrowed(&entry.msg)
	};

	let reply_to = entry_to_msg_map
		.as_mut()
		.and_then(|map| map.get_if_exists(entry.reply_to.as_ref()));

	tracing::debug!("Sending {msg:?} to a sink with tag {tag:?}, replying to {reply_to:?}");
	Ok(sink.send(&msg, reply_to, tag).await?)
}

async fn mark_entry_as_read(
	entry_id: &EntryId,
	msg_id: Option<MessageId>,
	// source: Option<&mut dyn Source>, // TODO: this doesn't work. Why?
	source: Option<&mut Box<dyn Source>>,
	entry_to_msg_map: Option<&mut EntryToMsgMap>,
) -> Result<(), Error> {
	if let Some(mar) = source {
		tracing::debug!("Marking {entry_id:?} as read");
		mar.mark_as_read(entry_id).await?;
	}

	if let Some((msgid, map)) = msg_id.zip(entry_to_msg_map) {
		tracing::debug!("Associating entry {entry_id:?} with message {msgid:?}");
		map.insert(entry_id.clone(), msgid).await?;
	}

	Ok(())
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
