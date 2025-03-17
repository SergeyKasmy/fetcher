/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Sink`] that can be used to consume a composed [`Message`],
//! as well as the [`message`] module itself

pub mod message;

pub mod discord;
pub mod stdout;
pub mod telegram;

pub mod error;

pub use self::{discord::Discord, stdout::Stdout, telegram::Telegram};
pub use crate::exec::Exec;
use crate::{
	action::{Action, ActionContext},
	entry::{Entry, EntryId},
	error::FetcherError,
	external_save::ExternalSave,
	source::Source,
	task::entry_to_msg_map::EntryToMsgMap,
};

use self::{
	error::SinkError,
	message::{Message, MessageId},
};

use std::{borrow::Cow, collections::HashSet, fmt::Debug};

/// An async function that sends a message somewhere
pub trait Sink: Debug + Send + Sync {
	/// Send the message with an optional tag (usually represented as a hashtag)
	async fn send(
		&self,
		message: &Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError>;
}

pub(crate) struct SinkWrapper<S>(pub S);

impl<Si> Action for SinkWrapper<Si>
where
	Si: Sink,
{
	type Error = FetcherError;

	async fn apply<'a, So, E>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a, So, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		So: Source,
		E: ExternalSave,
	{
		let undeduped_len = entries.len();
		tracing::trace!("Entries to send before dedup: {undeduped_len}");

		let entries = remove_duplicates(entries);

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
				&self.0,
				ctx.entry_to_msg_map.as_deref_mut(),
				ctx.tag.as_deref(),
				entry,
			)
			.await?;

			if let Some(entry_id) = entry.id.as_ref() {
				mark_entry_as_read(
					entry_id,
					msg_id,
					ctx.source.as_deref_mut(),
					ctx.entry_to_msg_map.as_deref_mut(),
				)
				.await?;
			}
		}

		Ok(entries)
	}
}

#[tracing::instrument(level = "trace", skip_all, fields(entry_id = ?entry.id))]
async fn send_entry<'a, S, E>(
	sink: &S,
	mut entry_to_msg_map: Option<&'a mut EntryToMsgMap<E>>,
	tag: Option<&str>,
	entry: &Entry,
) -> Result<Option<MessageId>, FetcherError>
where
	S: Sink,
	E: ExternalSave,
{
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
		.as_deref_mut()
		.and_then(|map| map.get_if_exists(entry.reply_to.as_ref()));

	tracing::debug!("Sending {msg:?} to a sink with tag {tag:?}, replying to {reply_to:?}");
	Ok(sink.send(&msg, reply_to, tag).await?)
}

async fn mark_entry_as_read<'a, S, E>(
	entry_id: &EntryId,
	msg_id: Option<MessageId>,
	source: Option<&'a mut S>,
	entry_to_msg_map: Option<&'a mut EntryToMsgMap<E>>,
) -> Result<(), FetcherError>
where
	S: Source + ?Sized,
	E: ExternalSave,
{
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
