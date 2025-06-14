/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Sink`] that can be used to consume a composed [`Message`],
//! as well as the [`message`] module itself

pub mod error;
pub mod message;

pub mod stdout;

pub use self::{message::Message, stdout::Stdout};
pub use crate::exec::Exec;

#[cfg(feature = "sink-telegram")]
pub mod telegram;
#[cfg(feature = "sink-telegram")]
pub use self::telegram::Telegram;

#[cfg(feature = "sink-discord")]
pub mod discord;
#[cfg(feature = "sink-discord")]
pub use self::discord::Discord;

use self::{error::SinkError, message::MessageId};
use crate::{
	actions::{Action, ActionContext, ActionResult},
	actres_try,
	entry::{Entry, EntryId},
	error::FetcherError,
	external_save::ExternalSave,
	maybe_send::{MaybeSend, MaybeSendSync},
	sources::{Source, error::SourceError},
	task::entry_to_msg_map::EntryToMsgMap,
};

use std::{borrow::Cow, collections::HashSet, convert::Infallible};

/// Adapter of [`Action`] tailored for handling composed messages.
///
/// Each message of each entry is passed to [`Sink::send`],
/// and if it returns `Ok`, the [`EntryId`] is automatically marked as read
/// and the returned [`MessageId`], if any, is added to the [`EntryToMsgMap`].
pub trait Sink: MaybeSendSync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<SinkError>;

	/// Sends the message with an optional tag.
	///
	/// The tag is often represented as a hashtag.
	///
	/// If the message is a reply to a different older already sent message,
	/// its [`MessageId`] is also passed. If supported, the sink might mark the current message
	/// as a reply to the older one.
	///
	/// # Returns
	/// A result that contains either `Some(MessageId)` if the sink supports [`MessageIds`](`MessageId`),
	/// or `None` if it doesn't.
	///
	/// The ID is currently only used for replies, so it's fine to return `None` if replies aren't used anyways.
	fn send(
		&mut self,
		message: &Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> impl Future<Output = Result<Option<MessageId>, Self::Err>> + MaybeSend;
}

/// Adapt a [`Sink`] to implement [`Action`] by applying [`Sink::send`] to each entry's message
pub struct SinkAction<S>(pub S);

impl<S: Sink> Sink for &mut S {
	type Err = S::Err;

	async fn send(
		&mut self,
		message: &Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, Self::Err> {
		(*self).send(message, reply_to, tag).await
	}
}

impl Sink for () {
	type Err = Infallible;

	async fn send(
		&mut self,
		_message: &Message,
		_reply_to: Option<&MessageId>,
		_tag: Option<&str>,
	) -> Result<Option<MessageId>, Self::Err> {
		Ok(None)
	}
}

impl Sink for Infallible {
	type Err = Infallible;

	async fn send(
		&mut self,
		_message: &Message,
		_reply_to: Option<&MessageId>,
		_tag: Option<&str>,
	) -> Result<Option<MessageId>, Self::Err> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl Sink for ! {
	type Err = !;

	async fn send(
		&mut self,
		_message: &Message,
		_reply_to: Option<&MessageId>,
		_tag: Option<&str>,
	) -> Result<Option<MessageId>, Self::Err> {
		match *self {}
	}
}

impl<S> Sink for Option<S>
where
	S: Sink,
{
	type Err = S::Err;

	async fn send(
		&mut self,
		message: &Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, Self::Err> {
		let Some(inner) = self else {
			return Ok(None);
		};

		inner.send(message, reply_to, tag).await
	}
}

impl<Si> Action for SinkAction<Si>
where
	Si: Sink,
{
	type Err = FetcherError;

	async fn apply<So, E>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'_, So, E>,
	) -> ActionResult<Self::Err>
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

		if !entries.is_empty() {
			tracing::trace!("Sending entries: {entries:#?}");
		}

		// entries should be sorted newest to oldest but we should send oldest first
		// TODO: should they be assumed to be sorted the other way instead?
		for entry in entries.iter().rev() {
			let msg_id = actres_try!(
				send_entry(
					&mut self.0,
					entry,
					ctx.entry_to_msg_map.as_deref_mut(),
					ctx.tag,
				)
				.await
			);

			if let Some(entry_id) = entry.id.as_ref() {
				actres_try!(
					mark_entry_as_read(
						entry_id,
						msg_id,
						ctx.source.as_deref_mut(),
						ctx.entry_to_msg_map.as_deref_mut(),
					)
					.await
				);
			}
		}

		tracing::trace!("Done sending entries");

		ActionResult::Ok(entries)
	}
}

#[tracing::instrument(level = "trace", skip_all, fields(entry_id = ?entry.id))]
async fn send_entry<'a, S, E>(
	sink: &mut S,
	entry: &Entry,
	mut entry_to_msg_map: Option<&'a mut EntryToMsgMap<E>>,
	tag: Option<&str>,
) -> Result<Option<MessageId>, FetcherError>
where
	S: Sink,
	E: ExternalSave,
{
	tracing::trace!("Sending entry");

	// send message if it isn't empty or raw_contents of they aren't
	let msg = match (entry.msg.is_empty(), &entry.raw_contents) {
		(false, _) => Cow::Borrowed(&entry.msg),
		(true, Some(raw_contents)) => {
			tracing::debug!("Message is empty, setting message body to raw_contents instead");

			Cow::Owned(Message {
				body: Some(raw_contents.clone()),
				..entry.msg.clone()
			})
		}
		_ => return Ok(None),
	};

	let reply_to = entry_to_msg_map
		.as_deref_mut()
		.and_then(|map| map.get_if_exists(entry.reply_to.as_ref()));

	tracing::debug!("Sending {msg:?} to a sink with tag {tag:?}, replying to {reply_to:?}");

	sink.send(&msg, reply_to, tag)
		.await
		.map_err(|e| FetcherError::from(e.into()))
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
		mar.mark_as_read(entry_id)
			.await
			.map_err(|e| SourceError::MarkAsRead(e.into()))?;
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
