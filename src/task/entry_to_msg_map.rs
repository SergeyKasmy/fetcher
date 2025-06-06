/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`EntryToMsgMap`]

use std::{collections::HashMap, convert::Infallible};

use crate::{
	entry::EntryId, error::FetcherError, external_save::ExternalSave, sinks::message::MessageId,
};

/// Map [`entries`][entry] to [`messages`][message]
///
/// [entry]: crate::entry::Entry
/// [message]: crate::sinks::message::Message
#[derive(Clone, Debug)]
pub struct EntryToMsgMap<E> {
	/// External save location for that map.
	/// It's called every time on [`Self::insert()`]
	pub external_save: Option<E>,

	map: HashMap<EntryId, MessageId>,
}

impl<E> EntryToMsgMap<E> {
	/// Creates a new empty map but with [`Self::external_save`] set to `external_save`.
	/// Use [`EntryToMsgMap::without_external_saver()`] if you don't want to set [`Self::external_save`]
	#[must_use]
	pub fn new(external_save: E) -> Self {
		Self {
			external_save: Some(external_save),
			map: HashMap::new(),
		}
	}

	/// Creates a new [`EntryToMsgMap`] with the provided `map` and `external_save` parameters
	#[must_use]
	pub fn new_with_map(map: HashMap<EntryId, MessageId>, external_save: E) -> Self {
		Self {
			external_save: Some(external_save),
			map,
		}
	}

	/// Creates a new instance of [`EntryToMsgMap`] without an external saver.
	/// This isn't very useful as all state will be lost when the program restarts.
	pub fn without_external_saver() -> Self {
		Self {
			external_save: None,
			map: HashMap::new(),
		}
	}

	/// Gets the [`MessageId`] corresponding to the provided [`EntryId`]
	#[must_use]
	pub fn get(&self, eid: &EntryId) -> Option<&MessageId> {
		self.map.get(eid)
	}

	/// Gets the [`MessageId`] corresponding to the provided [`EntryId`] if it exists
	#[must_use]
	pub fn get_if_exists(&self, eid: Option<&EntryId>) -> Option<&MessageId> {
		eid.and_then(|eid| self.map.get(eid))
	}
}

impl<E> EntryToMsgMap<E>
where
	E: ExternalSave,
{
	/// Insert a mapping from [`EntryId`] `eid` to [`MessageId`] `msgid` and save that externally
	///
	/// # Errors
	/// if external save has failed
	pub async fn insert(&mut self, eid: EntryId, msgid: MessageId) -> Result<(), FetcherError> {
		self.map.insert(eid, msgid);
		if let Some(ext_save) = &mut self.external_save {
			ext_save
				.save_entry_to_msg_map(&self.map)
				.await
				.map_err(FetcherError::ExternalSave)?;
		}

		Ok(())
	}
}

impl Default for EntryToMsgMap<Infallible> {
	fn default() -> Self {
		Self {
			external_save: None,
			map: HashMap::default(),
		}
	}
}
