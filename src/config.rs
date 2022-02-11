/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;
use teloxide::types::ChatId;
use url::Url;

use crate::{
	error::{Error, Result},
	settings, sink,
	source::{
		self,
		email::{Filters, ViewMode},
	},
};

#[derive(Deserialize, Debug)]
pub struct Telegram {
	chat_id: ChatId,
}

impl TryFrom<Telegram> for sink::Telegram {
	type Error = Error;

	fn try_from(v: Telegram) -> Result<Self> {
		Ok(sink::Telegram::new(settings::telegram()?, v.chat_id))
	}
}

#[derive(Deserialize)]
pub struct Email {
	imap: String,
	email: String,
	auth: EmailAuth,
	filters: Filters,
	view_mode: ViewMode,
	footer: Option<String>,
}

#[derive(Deserialize, Debug)]
pub enum EmailAuth {
	GoogleOAuth2,
	Password,
}

impl TryFrom<Email> for source::Email {
	type Error = Error;

	fn try_from(v: Email) -> Result<Self> {
		Ok(match v.auth {
			EmailAuth::GoogleOAuth2 => source::Email::with_google_oauth2(
				v.imap,
				v.email,
				settings::google_oauth2()?,
				v.filters,
				v.view_mode,
				v.footer,
			)?,
			EmailAuth::Password => source::Email::with_password(
				v.imap,
				v.email,
				settings::google_password()?,
				v.filters,
				v.view_mode,
				v.footer,
			),
		})
	}
}

#[derive(Deserialize)]
pub struct Twitter {
	pretty_name: String,
	handle: String,
	filter: Vec<String>,
}

impl TryFrom<Twitter> for source::Twitter {
	type Error = Error;

	fn try_from(v: Twitter) -> Result<Self> {
		let (api_key, api_secret) = settings::twitter()?;

		source::Twitter::new(v.pretty_name, v.handle, api_key, api_secret, v.filter)
	}
}

#[derive(Deserialize)]
pub struct Rss {
	url: Url,
}

impl From<Rss> for source::Rss {
	fn from(v: Rss) -> Self {
		source::Rss::new(v.url.to_string())
	}
}
