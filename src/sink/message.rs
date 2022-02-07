/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use url::Url;

pub enum Media {
	Photo(Url),
	Video(Url),
}

pub struct Message {
	pub text: String,
	pub media: Option<Vec<Media>>,
}

impl std::fmt::Debug for Message {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Message")
			.field("text", &self.text)
			.field("media.is_some()", &self.media.is_some())
			.finish()
	}
}
