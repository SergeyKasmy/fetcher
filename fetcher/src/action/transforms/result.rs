/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains everything needed to contruct a new [`Entry`] (via [`TransformedEntry`]) and [`Message`] (via [`TransformedMessage`]) after parsing, optionally using previous [`Entry's`](`Entry`) data if requested

use crate::{
	entry::{Entry, EntryId},
	sinks::message::{Media, Message},
};

use url::Url;

/// An [`Entry`] mirror that can be converted to [`Entry`] but whose fields can be chosen to inherit old entry's values on [`None`]
/// Refer to [`Entry`] for more docs on itself and each field
#[expect(
	missing_docs,
	reason = "a mirror of Entry struct, refer to Entry for docs"
)]
#[derive(Default, Debug)]
pub struct TransformedEntry {
	pub id: TransformResult<EntryId>,
	pub reply_to: TransformResult<EntryId>,
	pub raw_contents: TransformResult<String>,
	pub msg: TransformedMessage,
}

/// A [`Message`] mirror that can be converted to [`Message`] but whose fields can be chosen to inherit old message's values on [`None`]
/// Refer to [`Message`] for more docs on itself and each field
#[expect(
	missing_docs,
	reason = "a mirror of Message struct, refer to Message for docs"
)]
#[derive(Default, Debug)]
pub struct TransformedMessage {
	pub title: TransformResult<String>,
	pub body: TransformResult<String>,
	pub link: TransformResult<Url>,
	pub media: TransformResult<Vec<Media>>,
}

/// Specify whether to use previous/old, empty, or a new value
#[derive(Debug)]
pub enum TransformResult<T> {
	/// Keep the previous value
	Previous,

	/// Remove this value / make it empty
	Empty,

	/// Replace the value with this new value
	New(T),
}

/// Extension methods on [`Option<T>`] to transform [`Some(T)`] into [`TransformResult::New(T)`] and [`None`] into either [`TransformResult::Previous`] or [`TransformResult::Empty`]
pub trait OptionUnwrapTransformResultExt<T> {
	/// Transform [`Some(T)`] into [`TransformResult::New(T)`] and [`None`] into [`TransformResult::Previous`]
	fn unwrap_or_prev(self) -> TransformResult<T>;

	/// Transform [`Some(T)`] into [`TransformResult::New(T)`] and [`None`] into [`TransformResult::Empty`]
	fn unwrap_or_empty(self) -> TransformResult<T>;
}

impl TransformedEntry {
	/// Transform [`TransformedEntry`] into a new [`Entry`], using `old_entry`'s fields as fallback if needed
	#[must_use]
	pub fn into_entry(self, old_entry: &Entry) -> Entry {
		Entry {
			id: self.id.get(|| old_entry.id.clone()),
			reply_to: self.reply_to.get(|| old_entry.reply_to.clone()),
			raw_contents: self.raw_contents.get(|| old_entry.raw_contents.clone()),
			msg: self.msg.into_message(&old_entry.msg),
		}
	}
}

impl TransformedMessage {
	/// Transform [`TransformedMessage`] into a new [`Message`], using `old_msg`'s fields as fallback if needed
	#[must_use]
	pub fn into_message(self, old_msg: &Message) -> Message {
		Message {
			title: self.title.get(|| old_msg.title.clone()),
			body: self.body.get(|| old_msg.body.clone()),
			link: self.link.get(|| old_msg.link.clone()),
			media: self.media.get(|| old_msg.media.clone()),
		}
	}
}

impl<T> TransformResult<T> {
	/// Combine new value with the old value using new value's merge stradegy
	pub fn get<F>(self, old_val: F) -> Option<T>
	where
		F: FnOnce() -> Option<T>,
	{
		match self {
			Self::Previous => old_val(),
			Self::Empty => None,
			Self::New(val) => Some(val),
		}
	}

	pub fn map<F, U>(self, f: F) -> TransformResult<U>
	where
		F: FnOnce(T) -> U,
	{
		match self {
			TransformResult::Previous => TransformResult::Previous,
			TransformResult::Empty => TransformResult::Empty,
			TransformResult::New(t) => TransformResult::New(f(t)),
		}
	}

	pub fn try_map<F, U, E>(self, f: F) -> Result<TransformResult<U>, E>
	where
		F: FnOnce(T) -> Result<U, E>,
	{
		Ok(match self {
			TransformResult::Previous => TransformResult::Previous,
			TransformResult::Empty => TransformResult::Empty,
			TransformResult::New(t) => TransformResult::New(f(t)?),
		})
	}
}

impl<T> Default for TransformResult<T> {
	fn default() -> Self {
		Self::Previous
	}
}

impl<T> OptionUnwrapTransformResultExt<T> for Option<T> {
	fn unwrap_or_prev(self) -> TransformResult<T> {
		self.map_or_else(|| TransformResult::Previous, TransformResult::New)
	}

	fn unwrap_or_empty(self) -> TransformResult<T> {
		self.map_or_else(|| TransformResult::Empty, TransformResult::New)
	}
}
