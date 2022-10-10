/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Message`] and [`Media`]

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

/// A link to some kind of external media
#[derive(Clone)]
pub enum Media {
	/// A link to a photo
	Photo(Url),
	/// A link to a video
	Video(Url),
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
