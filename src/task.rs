/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic block of [`fetcher`](`crate`) that is a [`Task`].

mod disabled_task;
mod opaque_task;
mod task_group;

pub mod entry_to_msg_map;

pub use self::disabled_task::DisabledTask;
pub use self::opaque_task::OpaqueTask;
pub use self::task_group::TaskGroup;

use self::entry_to_msg_map::EntryToMsgMap;
use crate::{
	StaticStr,
	actions::{Action, ActionContext},
	entry::Entry,
	error::FetcherError,
	external_save::ExternalSave,
	sources::Source,
};

/// A core primitive of [`fetcher`](`crate`). A single instance of a data pipeline.
///
/// Runs the data fetched from a [`Source`] through the pipeline ([`Task::action`])
#[derive(bon::Builder, Debug)]
pub struct Task<S, A, E> {
	/// Name of the task
	#[builder(start_fn, into)]
	pub name: StaticStr,

	/// Map of an entry (by [`EntryId`](`crate::entry::EntryId`)) to a sent message (by [`MessageId`](`crate::sinks::message::MessageId`)).
	///
	/// Sinks supporting replies can make the current message a reply to an older one.
	#[builder(field)]
	pub entry_to_msg_map: Option<EntryToMsgMap<E>>,

	/// Optional tag that a [`Sink`](`crate::sinks::Sink`) may put near a message body to differentiate this task from others that may be similar.
	///
	/// For example, messages from different task that are sent to the same sink can be differentiated using this adjecent tag.
	#[builder(into)]
	pub tag: Option<StaticStr>,

	/// Source where to fetch the data from.
	///
	/// Also used to mark the entry as read after it's been sent.
	pub source: Option<S>,

	/// Pipeline (in other words, a list of actions) which the data received from the source is run through
	pub action: Option<A>,
}

impl<S, A, E> Task<S, A, E>
where
	S: Source,
	A: Action,
	E: ExternalSave,
{
	/// Run a task once to completion
	///
	/// # Errors
	/// Errors if any part of the pipeline (source -> actions) failed,
	/// if the [`ReadFilter`] failed,
	/// or if the [`ExternalSave`] implementation caused the [`EntryToMsgMap`] to return an error.
	#[expect(clippy::same_name_method, reason = "can't think of a better name")] // if any come up, I'd be fine to replace it
	#[tracing::instrument(skip(self), fields(name = %self.name))]
	async fn run(&mut self) -> Result<(), FetcherError> {
		tracing::trace!("Running task");

		let raw = match &mut self.source {
			Some(source) => source.fetch().await?,
			None => vec![Entry::default()], // return just an empty entry if there is no source
		};

		tracing::debug!("Got {} raw entries from the sources", raw.len());
		tracing::trace!("Raw entries: {raw:#?}");

		if let Some(action) = &mut self.action {
			let ctx = ActionContext {
				source: self.source.as_mut(),
				entry_to_msg_map: self.entry_to_msg_map.as_mut(),
				tag: self.tag.as_deref(),
			};
			action.apply(raw, ctx).await.map_err(Into::into)?;
		}

		Ok(())
	}
}

impl<S, A, E> OpaqueTask for Task<S, A, E>
where
	S: Source,
	A: Action,
	E: ExternalSave,
{
	async fn run(&mut self) -> Result<(), FetcherError> {
		Task::run(self).await
	}
}

impl<S, A, State: task_builder::State> TaskBuilder<S, A, (), State> {
	/// Disables [`Task::entry_to_msg_map`].
	///
	/// Even though [`Task::entry_to_msg_map`] is optional, the generic still needs to be specified.
	/// This method specifies the generic as [`()`] and sets [`Task::entry_to_msg_map`] to `None`.
	pub fn no_entry_to_msg_map(mut self) -> Self {
		self.entry_to_msg_map = None;
		self
	}

	/// Builds the task while disabling the [`Task::entry_to_msg_map`] via [`TaskBuilder::no_entry_to_msg_map`].
	pub fn build_without_replies(self) -> Task<S, A, ()> {
		self.no_entry_to_msg_map().build()
	}
}
