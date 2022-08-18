/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::Debug;

use url::Url;

#[derive(Clone, Default)]
pub struct Message {
	pub title: Option<String>,
	pub body: String,
	pub link: Option<Url>,
	pub media: Option<Vec<Media>>,
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

#[derive(Clone)]
pub enum Media {
	Photo(Url),
	Video(Url),
}

impl Debug for Media {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Photo(x) => f.debug_tuple("Photo").field(&x.as_str()).finish(),
			Self::Video(x) => f.debug_tuple("Video").field(&x.as_str()).finish(),
		}
	}
}
