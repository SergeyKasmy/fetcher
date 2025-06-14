/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Message`] and [`Media`]

// TODO: make pub, improve docs and API first
pub(crate) mod length_limiter;

use std::{borrow::Cow, fmt::Debug};

use non_non_full::NonEmptyVec;

use crate::safe_slice::SafeSliceUntilExt;

/// The finalized and composed message meant to be sent to a sink
#[derive(PartialEq, Eq, Clone, Default, bon::Builder)]
pub struct Message {
	/// title of the message
	pub title: Option<String>,

	// TODO: add support for rich text. If the sink doesn't support it, it can just be stripped
	/// body of the message
	pub body: Option<String>,

	/// a url to the full contents or source of the message
	pub link: Option<String>,
	/// a list of photos or videos included in the message. They are usually attached to the message itself if the sink supports it. Otherwise they may be left as links
	pub media: Option<NonEmptyVec<Media>>,
}

// TODO: the type of the message id could be probably stored as an associated type inside Sink
// This would allow to specify what types support message ids and which ones don't, as well as avoid conversions between different int types or even str
/// An id of a sent message
#[derive(Clone, Copy, Debug)]
pub struct MessageId(pub i64);

// TODO: rename photo to image mb?
/// A link to some kind of external media
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Media {
	/// A link to a photo
	Photo(String),
	/// A link to a video
	Video(String),
}

impl Message {
	/// Check if the message is entirely empty. Even a single media attachment will mark this message as not empty
	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.title.is_none() && self.body.is_none() && self.link.is_none() && self.media.is_none()
	}
}

impl Debug for Message {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		#[expect(clippy::ptr_arg, reason = "the same as Option::map expected signature")]
		fn limit_max_len_to_250b(s: &String) -> Cow<'_, str> {
			s.pretty_slice_until(250)
		}

		f.debug_struct("Message")
			.field("title", &self.title.as_ref().map(limit_max_len_to_250b))
			.field("body", &self.body.as_ref().map(limit_max_len_to_250b))
			.field("link", &self.link.as_ref().map(limit_max_len_to_250b))
			.field("media", &self.media)
			.finish()
	}
}

impl From<i64> for MessageId {
	fn from(value: i64) -> Self {
		Self(value)
	}
}

impl<S> From<MessageBuilder<S>> for Message
where
	S: message_builder::IsComplete,
{
	fn from(value: MessageBuilder<S>) -> Self {
		value.build()
	}
}
