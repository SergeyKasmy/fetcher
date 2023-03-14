/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Message`] and [`Media`]

pub(crate) mod length_limiter;

use std::fmt::Debug;
use url::Url;

/// The finalized and composed message meant to be sent to a sink
#[derive(Clone, Default)]
pub struct Message {
	/// title of the message
	pub title: Option<String>,
	/// body of the message
	pub body: Option<String>,
	/// a url to the full contents or source of the message
	pub link: Option<Url>,
	/// a list of photos or videos included in the message. They are usually attached to the message itself if the sink supports it. Otherwise they may be left as links
	pub media: Option<Vec<Media>>,
}

// TODO: the type of the message id could be probably stored as an associated type inside Sink
// This would allow to specify what types support message ids and which ones don't, as well as avoid conversions between different int types or even str
/// An id of a sent message
#[derive(Clone, Copy, Debug)]
pub struct MessageId(pub i64);

// TODO: rename photo to image mb?
/// A link to some kind of external media
#[derive(Clone)]
pub enum Media {
	/// A link to a photo
	Photo(Url),
	/// A link to a video
	Video(Url),
}

impl Message {
	/// Check if the message is entirely empty. Even a single media attachment will mark this message as not empty
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.title.is_none() && self.body.is_none() && self.link.is_none() && self.media.is_none()
	}
}

impl From<i64> for MessageId {
	fn from(value: i64) -> Self {
		Self(value)
	}
}

impl Debug for Message {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Message")
			.field("title", &self.title)
			.field("body", &self.body)
			.field("link", &self.link.as_ref().map(Url::as_str))
			.field("media", &self.media)
			.finish()
	}
}

impl Debug for Media {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Photo(x) => f.debug_tuple("Photo").field(&x.as_str()).finish(),
			Self::Video(x) => f.debug_tuple("Video").field(&x.as_str()).finish(),
		}
	}
}
