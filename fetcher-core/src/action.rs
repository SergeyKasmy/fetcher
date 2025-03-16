/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all [`Actions`](`Action`) that a list of [`Entry`]'s can be run through to view/modify/filter it out

pub mod filter;
pub mod transform;

use std::{borrow::Cow, collections::HashSet};

use crate::{
	entry::{Entry, EntryId},
	error::FetcherError,
	sink::{
		Sink,
		message::{Message, MessageId},
	},
	source::Source,
	task::entry_to_msg_map::EntryToMsgMap,
};

use self::{
	filter::Filter,
	transform::{Transform, error::TransformError},
};

pub struct ActionContext<'a> {
	pub source: Option<&'a mut dyn Source>,
	pub entry_to_msg_map: Option<&'a mut EntryToMsgMap>,
	pub tag: Option<&'a str>,
}

pub trait Action {
	type Error: Into<FetcherError>;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error>;
}

pub struct FilterWrapper<F>(pub F);

impl<F> Action for FilterWrapper<F>
where
	F: Filter,
{
	type Error = !;

	async fn apply(
		&mut self,
		mut entries: Vec<Entry>,
		_ctx: ActionContext<'_>,
	) -> Result<Vec<Entry>, Self::Error> {
		self.0.filter(&mut entries).await;

		Ok(entries)
	}
}

pub struct TransformWrapper<T>(pub T);

impl<T> Action for TransformWrapper<T>
where
	T: Transform,
{
	type Error = TransformError;

	async fn apply(
		&mut self,
		entries: Vec<Entry>,
		_ctx: ActionContext<'_>,
	) -> Result<Vec<Entry>, Self::Error> {
		let mut transformed_entries = Vec::new();

		for entry in entries {
			transformed_entries.extend(self.0.transform(entry).await?);
		}

		Ok(transformed_entries)
	}
}

pub struct SinkWrapper<S>(pub S);

impl<S> Action for SinkWrapper<S>
where
	S: Sink,
{
	type Error = FetcherError;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error> {
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

macro_rules! reborrow_ctx {
	($ctx:expr) => {{
		let ctx = $ctx;
		ActionContext {
			source: ctx.source.as_deref_mut(),
			entry_to_msg_map: ctx.entry_to_msg_map.as_deref_mut(),
			tag: ctx.tag.as_deref(),
		}
	}};
}

impl<A1> Action for (A1,)
where
	A1: Action,
{
	type Error = FetcherError;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error> {
		self.0.apply(entries, context).await.map_err(Into::into)
	}
}

impl<A1, A2> Action for (A1, A2)
where
	A1: Action,
	A2: Action,
{
	type Error = FetcherError;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error> {
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;
		self.1.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3> Action for (A1, A2, A3)
where
	A1: Action,
	A2: Action,
	A3: Action,
{
	type Error = FetcherError;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error> {
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.2.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3, A4> Action for (A1, A2, A3, A4)
where
	A1: Action,
	A2: Action,
	A3: Action,
	A4: Action,
{
	type Error = FetcherError;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error> {
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.2
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.3.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3, A4, A5> Action for (A1, A2, A3, A4, A5)
where
	A1: Action,
	A2: Action,
	A3: Action,
	A4: Action,
	A5: Action,
{
	type Error = FetcherError;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error> {
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.2
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.3
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.4.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3, A4, A5, A6> Action for (A1, A2, A3, A4, A5, A6)
where
	A1: Action,
	A2: Action,
	A3: Action,
	A4: Action,
	A5: Action,
	A6: Action,
{
	type Error = FetcherError;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error> {
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.2
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.3
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.4
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.5.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3, A4, A5, A6, A7> Action for (A1, A2, A3, A4, A5, A6, A7)
where
	A1: Action,
	A2: Action,
	A3: Action,
	A4: Action,
	A5: Action,
	A6: Action,
	A7: Action,
{
	type Error = FetcherError;

	async fn apply<'a>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a>,
	) -> Result<Vec<Entry>, Self::Error> {
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.2
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.3
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.4
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.5
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.6.apply(entries, ctx).await.map_err(Into::into)
	}
}

#[tracing::instrument(level = "trace", skip_all, fields(entry_id = ?entry.id))]
async fn send_entry<'a>(
	sink: &dyn Sink,
	mut entry_to_msg_map: Option<&'a mut EntryToMsgMap>,
	tag: Option<&str>,
	entry: &Entry,
) -> Result<Option<MessageId>, FetcherError> {
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
async fn mark_entry_as_read<'a>(
	entry_id: &EntryId,
	msg_id: Option<MessageId>,
	// source: Option<&mut dyn Source>, // TODO: this doesn't work. Why?
	source: Option<&'a mut dyn Source>,
	entry_to_msg_map: Option<&'a mut EntryToMsgMap>,
) -> Result<(), FetcherError> {
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
