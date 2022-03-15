/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use url::Url;

#[derive(Default, Debug)]
pub struct Message {
	pub title: Option<String>,
	pub body: String,
	pub link: Option<Link>,
	pub media: Option<Vec<Media>>,
}

#[derive(Debug)]
pub struct Link {
	pub url: Url,
	pub loc: LinkLocation,
}

/// Either embed the link into the title or put it as a separate "Link" button at the botton of the message.
/// `PreferTitle` falls back to `Bottom` if Message.title is None
#[derive(Debug)]
pub enum LinkLocation {
	PreferTitle,
	Bottom,
}

#[derive(Debug)]
pub enum Media {
	Photo(Url),
	Video(Url),
}
