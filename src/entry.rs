/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use crate::{read_filter::Id, sink::Message};

// TODO: add message history via responce id -> message id hashmap
// TODO: add pretty name/hashtag and link here instead of doing it manually
#[derive(Debug)]
pub struct Entry {
	pub id: String, // TODO: add date id type
	pub msg: Message,
}

impl Id for Entry {
	fn id(&self) -> &str {
		self.id.as_str()
	}
}
