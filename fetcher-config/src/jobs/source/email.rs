/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod auth;
mod filters;
mod view_mode;

use serde::{Deserialize, Serialize};

use self::{auth::Auth, filters::Filters, view_mode::ViewMode};
use crate::{
	jobs::external_data::{ExternalDataResult, ProvideExternalData},
	Error as ConfigError,
};
use fetcher_core::source::{Email as CEmail, WithCustomRF as CWithCustomRF};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Email {
	imap: Option<String>,
	email: String,
	auth: Auth,
	filters: Filters,
	view_mode: ViewMode,
}

impl Email {
	pub fn parse(self, external: &dyn ProvideExternalData) -> Result<CWithCustomRF, ConfigError> {
		let email_source = match self.auth {
			Auth::GoogleOAuth2 => {
				let oauth = match external.google_oauth2() {
					ExternalDataResult::Ok(v) => v,
					ExternalDataResult::Unavailable => {
						return Err(ConfigError::GoogleOAuth2TokenMissing);
					}
					ExternalDataResult::Err(e) => return Err(e.into()),
				};

				CEmail::with_google_oauth2(
					self.email,
					oauth,
					self.filters.parse(),
					self.view_mode.parse(),
				)
			}
			Auth::Password => {
				let passwd = match external.email_password() {
					ExternalDataResult::Ok(v) => v,
					ExternalDataResult::Unavailable => {
						return Err(ConfigError::EmailPasswordMissing);
					}
					ExternalDataResult::Err(e) => return Err(e.into()),
				};

				CEmail::with_password(
					self.imap.ok_or(ConfigError::EmailImapFieldMissing)?,
					self.email,
					passwd,
					self.filters.parse(),
					self.view_mode.parse(),
				)
			}
		};

		Ok(CWithCustomRF::Email(email_source))
	}
}
