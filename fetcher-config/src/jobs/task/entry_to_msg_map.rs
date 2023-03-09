/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use fetcher_core::{entry::EntryId as CEntryId, sink::message::MessageId as CMessageId};

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Debug)]
#[serde(transparent)]
pub struct EntryId(pub String);

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct MessageId(pub i64);

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct EntryToMsgMap(pub HashMap<EntryId, MessageId>);

impl EntryId {
	pub fn parse(self) -> CEntryId {
		self.0.into()
	}

	pub fn unparse(eid: CEntryId) -> Self {
		Self(eid.0)
	}
}

impl MessageId {
	pub fn parse(self) -> CMessageId {
		self.0.into()
	}

	pub fn unparse(msgid: CMessageId) -> Self {
		Self(msgid.0)
	}
}

impl EntryToMsgMap {
	pub fn parse(self) -> HashMap<CEntryId, CMessageId> {
		self.0
			.into_iter()
			.map(|(eid, msgid)| (eid.parse(), msgid.parse()))
			.collect()
	}

	pub fn unparse(map: HashMap<CEntryId, CMessageId>) -> Self {
		Self(
			map.into_iter()
				.map(|(eid, msgid)| (EntryId::unparse(eid), MessageId::unparse(msgid)))
				.collect(),
		)
	}
}
