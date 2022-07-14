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

use serde::{Deserialize, Serialize};

use self::auth::Auth;
use self::filters::Filters;
use self::view_mode::ViewMode;
use crate::config::DataSettings;
use crate::error::config::Error as ConfigError;
use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
pub(crate) struct Email {
	imap: String,
	email: String,
	auth: Auth,
	filters: Filters,
	view_mode: ViewMode,
	footer: Option<String>,
}

impl Email {
	pub(crate) fn parse(self, settings: &DataSettings) -> Result<source::Email, ConfigError> {
		Ok(match self.auth {
			Auth::GoogleOAuth2 => source::Email::with_google_oauth2(
				self.imap,
				self.email,
				settings
					.google_oauth2
					.as_ref()
					.cloned()
					.ok_or(ConfigError::GoogleOAuth2TokenMissing)?,
				self.filters.parse(),
				self.view_mode.parse(),
				self.footer,
			),
			Auth::Password => source::Email::with_password(
				self.imap,
				self.email,
				settings
					.email_password
					.as_ref()
					.cloned()
					.ok_or(ConfigError::EmailPasswordMissing)?,
				self.filters.parse(),
				self.view_mode.parse(),
				self.footer,
			),
		})
	}
}
