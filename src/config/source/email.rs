/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

mod auth;
mod filters;
mod view_mode;

use serde::Deserialize;

use self::auth::Auth;
use self::filters::Filters;
use self::view_mode::ViewMode;
use crate::{error::Result, settings, source};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Email {
	imap: String,
	email: String,
	auth: Auth,
	filters: Filters,
	view_mode: ViewMode,
	footer: Option<String>,
}

impl Email {
	pub(crate) fn parse(self) -> Result<source::Email> {
		Ok(match self.auth {
			Auth::GoogleOAuth2 => source::Email::with_google_oauth2(
				self.imap,
				self.email,
				settings::google_oauth2()?,
				self.filters.parse(),
				self.view_mode.parse(),
				self.footer,
			)?,
			Auth::Password => source::Email::with_password(
				self.imap,
				self.email,
				settings::google_password()?,
				self.filters.parse(),
				self.view_mode.parse(),
				self.footer,
			),
		})
	}
}
