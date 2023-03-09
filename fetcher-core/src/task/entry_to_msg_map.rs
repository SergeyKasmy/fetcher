/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;

use crate::{entry::EntryId, error::Error, external_save::ExternalSave, sink::message::MessageId};

#[derive(Default, Debug)]
pub struct EntryToMsgMap {
	pub map: HashMap<EntryId, MessageId>,
	pub external_save: Option<Box<dyn ExternalSave>>,
}

impl EntryToMsgMap {
	pub fn new(external_save: Box<dyn ExternalSave>) -> Self {
		Self {
			map: HashMap::new(),
			external_save: Some(external_save),
		}
	}

	pub async fn insert(&mut self, eid: EntryId, msgid: MessageId) -> Result<(), Error> {
		self.map.insert(eid, msgid);
		if let Some(ext_save) = &mut self.external_save {
			ext_save
				.save_entry_to_msg_map(&self.map)
				.await
				.map_err(Error::ExternalSave)?;
		}

		Ok(())
	}

	pub fn get(&self, eid: &EntryId) -> Option<&MessageId> {
		self.map.get(eid)
	}

	pub fn get_if_exists(&self, eid: Option<&EntryId>) -> Option<&MessageId> {
		eid.and_then(|eid| self.map.get(eid))
	}
}
