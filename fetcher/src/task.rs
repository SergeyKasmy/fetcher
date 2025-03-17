/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic block of [`fetcher`](`crate`) that is a [`Task`]

pub mod entry_to_msg_map;

use self::entry_to_msg_map::EntryToMsgMap;
use crate::{
	action::{Action, ActionContext},
	entry::Entry,
	error::FetcherError,
	external_save::ExternalSave,
	source::Source,
};

/// A core primitive of [`fetcher`](`crate`).
///
/// Contains everything from a [`Source`] that allows to fetch some data, to a [`Sink`] that takes that data and sends it somewhere.
/// It also contains any transformators
#[derive(Debug)]
pub struct Task<S, A, E> {
	/// An optional tag that may be put near a message body to differentiate this task from others that may be similar
	pub tag: Option<String>,

	/// The source where to fetch some data from
	pub source: Option<S>,

	/// A list of optional transformators which to run the data received from the source through
	pub action: Option<A>,

	/// Map of an entry to a message. Used when an entry is a reply to an older entry to be able to show that as a message, too
	pub entry_to_msg_map: Option<EntryToMsgMap<E>>,
}

impl<S, A, E> Task<S, A, E>
where
	S: Source,
	A: Action,
	E: ExternalSave,
{
	/// Run a task (both the source and the sink part) once to completion
	///
	/// # Errors
	/// If there was an error fetching the data, sending the data, or saving what data was successfully sent to an external location
	#[tracing::instrument(skip(self))]
	pub async fn run(&mut self) -> Result<(), FetcherError> {
		tracing::trace!("Running task");

		let raw = match &mut self.source {
			Some(source) => source.fetch().await?,
			None => vec![Entry::default()], // return just an empty entry if there is no source
		};

		tracing::debug!("Got {} raw entries from the sources", raw.len());
		tracing::trace!("Raw entries: {raw:#?}");

		// self.process_entries(raw).await?;
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

pub trait OpaqueTask {
	async fn run(&mut self) -> Result<(), FetcherError>;
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
