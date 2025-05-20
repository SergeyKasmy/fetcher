/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic block of [`fetcher`](`crate`) that is a [`Task`]

mod task_group;

pub mod entry_to_msg_map;

pub use self::task_group::TaskGroup;

use self::entry_to_msg_map::EntryToMsgMap;
use crate::{
	StaticStr,
	action::{Action, ActionContext},
	entry::Entry,
	error::FetcherError,
	external_save::ExternalSave,
	maybe_send::{MaybeSend, MaybeSendSync},
	sources::Source,
};

/// A core primitive of [`fetcher`](`crate`).
///
/// Contains everything from a [`Source`] that allows to fetch some data, to a [`Sink`] that takes that data and sends it somewhere.
/// It also contains any transformators
#[derive(bon::Builder, Debug)]
pub struct Task<S, A, E> {
	#[builder(start_fn, into)]
	pub name: StaticStr,

	/// Map of an entry to a message. Used when an entry is a reply to an older entry to be able to show that as a message, too
	#[builder(field)]
	pub entry_to_msg_map: Option<EntryToMsgMap<E>>,

	/// An optional tag that may be put near a message body to differentiate this task from others that may be similar
	#[builder(into)]
	pub tag: Option<StaticStr>,

	/// The source where to fetch some data from
	pub source: Option<S>,

	/// A list of optional transformators which to run the data received from the source through
	pub action: Option<A>,
}

pub trait OpaqueTask: MaybeSendSync {
	fn run(&mut self) -> impl Future<Output = Result<(), FetcherError>> + MaybeSend;

	fn disable(self) -> DisabledTask<Self>
	where
		Self: Sized,
	{
		DisabledTask(self)
	}
}

impl<S, A, E> OpaqueTask for Task<S, A, E>
where
	S: Source,
	A: Action,
	E: ExternalSave,
{
	/// Run a task (both the source and the sink part) once to completion
	///
	/// # Errors
	/// If there was an error fetching the data, sending the data, or saving what data was successfully sent to an external location
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

impl OpaqueTask for () {
	async fn run(&mut self) -> Result<(), FetcherError> {
		Ok(())
	}
}

impl<T> OpaqueTask for Option<T>
where
	T: OpaqueTask,
{
	async fn run(&mut self) -> Result<(), FetcherError> {
		let Some(task) = self else {
			return Ok(());
		};

		task.run().await
	}
}

pub struct DisabledTask<T>(T);

impl<T: MaybeSendSync> OpaqueTask for DisabledTask<T> {
	async fn run(&mut self) -> Result<(), FetcherError> {
		Ok(())
	}
}

impl<S, A, State: task_builder::State> TaskBuilder<S, A, (), State> {
	pub fn no_entry_to_msg_map(mut self) -> Self {
		self.entry_to_msg_map = None;
		self
	}

	pub fn build_without_replies(self) -> Task<S, A, ()> {
		self.no_entry_to_msg_map().build()
	}
}
