/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;

use crate::{
	error::Error,
	error::Result,
	settings,
	source::{
		self,
		email::{Filters, ViewMode},
	},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Email {
	imap: String,
	email: String,
	auth: EmailAuth,
	filters: Filters,
	view_mode: ViewMode,
	footer: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum EmailAuth {
	#[serde(rename = "google_oauth2")]
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
