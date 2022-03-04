/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use url::Url;

#[derive(Debug)]
pub struct Message {
	pub text: String,
	pub media: Option<Vec<Media>>,
}

#[derive(Debug)]
pub enum Media {
	Photo(Url),
	Video(Url),
}
