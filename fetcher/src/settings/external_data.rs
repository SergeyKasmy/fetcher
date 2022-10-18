/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{context::StaticContext, data, read_filter};
use fetcher_config::tasks::external_data::{ExternalData, ExternalDataError};
use fetcher_core::read_filter::{Kind as ReadFilterKind, ReadFilter};

pub struct ExternalDataFromDataDir {
	pub cx: StaticContext,
}

impl ExternalData for ExternalDataFromDataDir {
	fn twitter_token(&self) -> Result<Option<(String, String)>, ExternalDataError> {
		data::twitter::get(self.cx)
	}

	fn google_oauth2(&self) -> Result<Option<fetcher_core::auth::Google>, ExternalDataError> {
		data::google_oauth2::get(self.cx)
	}

	fn email_password(&self) -> Result<Option<String>, ExternalDataError> {
		data::email_password::get(self.cx)
	}

	fn telegram_bot_token(&self) -> Result<Option<String>, ExternalDataError> {
		data::telegram::get(self.cx)
	}

	fn read_filter(
		&self,
		name: &str,
		expected_rf: ReadFilterKind,
	) -> Result<ReadFilter, ExternalDataError> {
		read_filter::get(name, expected_rf, self.cx)
	}
}
