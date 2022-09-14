/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
	entry::Entry,
	sink::{Media, Message},
};

use url::Url;

#[derive(Default, Debug)]
pub struct TransformedEntry {
	pub id: TransformResult<String>,
	pub raw_contents: TransformResult<String>,
	pub msg: TransformedMessage,
}

#[derive(Default, Debug)]
pub struct TransformedMessage {
	pub title: TransformResult<String>,
	pub body: TransformResult<String>,
	pub link: TransformResult<Url>,
	pub media: TransformResult<Vec<Media>>,
}

/// Previous: uses previous value if None, and a new one if Some
/// New: uses empty value if None, and a new one if Some
#[derive(Debug)]
pub enum TransformResult<T> {
	Old(Option<T>),
	New(Option<T>),
}

impl TransformedEntry {
	pub fn into_entry(self, old_entry: Entry) -> Entry {
		Entry {
			id: self.id.get(old_entry.id),
			raw_contents: self.raw_contents.get(old_entry.raw_contents),
			msg: self.msg.into_message(old_entry.msg),
		}
	}
}

impl TransformedMessage {
	pub fn into_message(self, old_msg: Message) -> Message {
		Message {
			title: self.title.get(old_msg.title),
			body: self.body.get(old_msg.body),
			link: self.link.get(old_msg.link),
			media: self.media.get(old_msg.media),
		}
	}
}

impl<T> TransformResult<T> {
	pub fn get(self, old_val: Option<T>) -> Option<T> {
		use TransformResult::{New, Old};

		match self {
			Old(val) => val.or(old_val),
			New(val) => val,
		}
	}
}

impl<T> Default for TransformResult<T> {
	fn default() -> Self {
		TransformResult::Old(None)
	}
}
