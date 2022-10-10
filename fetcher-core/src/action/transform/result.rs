/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains everything needed to contruct a new [`Entry`] (via [`TransformedEntry`]) and [`Message`] (via [`TransformedMessage`]) after parsing, optionally using previous [`Entry's`](`Entry`) data if requested

use crate::{
	entry::Entry,
	sink::{Media, Message},
};

use url::Url;

/// An [`Entry`] mirror that can be converted to [`Entry`] but whose fields can be chosen to inherit old entry's values on [`None`]
/// Refer to [`Entry`] for more docs on itself and each field
#[allow(missing_docs)]
#[derive(Default, Debug)]
pub struct TransformedEntry {
	pub id: TransformResult<String>,
	pub raw_contents: TransformResult<String>,
	pub msg: TransformedMessage,
}

/// A [`Message`] mirror that can be converted to [`Message`] but whose fields can be chosen to inherit old message's values on [`None`]
/// Refer to [`Message`] for more docs on itself and each field
#[allow(missing_docs)]
#[derive(Default, Debug)]
pub struct TransformedMessage {
	pub title: TransformResult<String>,
	pub body: TransformResult<String>,
	pub link: TransformResult<Url>,
	pub media: TransformResult<Vec<Media>>,
}

/// An [`Option`] wrapper that can specify what to replace the value with if it's [`None`]
#[derive(Debug)]
pub enum TransformResult<T> {
	/// Use previous value if None, and a new one if Some
	Old(Option<T>),
	/// Use empty value if None, and a new one if Some
	New(Option<T>),
}

impl TransformedEntry {
	/// Transform [`TransformedEntry`] into a new [`Entry`], using `old_entry`'s fields as fallback if needed
	#[must_use]
	pub fn into_entry(self, old_entry: Entry) -> Entry {
		Entry {
			id: self.id.get(old_entry.id),
			raw_contents: self.raw_contents.get(old_entry.raw_contents),
			msg: self.msg.into_message(old_entry.msg),
		}
	}
}

impl TransformedMessage {
	/// Transform [`TransformedMessage`] into a new [`Message`], using `old_msg`'s fields as fallback if needed
	#[must_use]
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
	/// Combine new value with the old value using new value's merge stradegy
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
