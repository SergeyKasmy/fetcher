/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod auth;
mod filters;
mod view_mode;

use serde::{Deserialize, Serialize};

use self::auth::Auth;
use self::filters::Filters;
use self::view_mode::ViewMode;
use crate::tasks::TaskSettings;
use crate::Error;
use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
pub struct Email {
	imap: Option<String>,
	email: String,
	auth: Auth,
	filters: Filters,
	view_mode: ViewMode,
	footer: Option<String>,
}

impl Email {
	pub fn parse(self, settings: &dyn TaskSettings) -> Result<source::Email, Error> {
		Ok(match self.auth {
			Auth::GoogleOAuth2 => source::Email::with_google_oauth2(
				self.email,
				settings
					.google_oauth2()?
					.ok_or(Error::GoogleOAuth2TokenMissing)?,
				self.filters.parse(),
				self.view_mode.parse(),
				self.footer,
			),
			Auth::Password => source::Email::with_password(
				self.imap.ok_or(Error::EmailImapFieldMissing)?,
				self.email,
				settings
					.email_password()?
					.ok_or(Error::EmailPasswordMissing)?,
				self.filters.parse(),
				self.view_mode.parse(),
				self.footer,
			),
		})
	}
}
