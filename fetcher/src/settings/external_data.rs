/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{context::StaticContext, data, read_filter};
use fetcher_config::jobs::external_data::{ExternalDataResult, ProvideExternalData};
use fetcher_core::read_filter::{Kind as ReadFilterKind, ReadFilter};

pub struct ExternalDataFromDataDir {
	pub cx: StaticContext,
}

impl ProvideExternalData for ExternalDataFromDataDir {
	fn twitter_token(&self) -> ExternalDataResult<(String, String)> {
		data::twitter::get(self.cx).into()
	}

	fn google_oauth2(&self) -> ExternalDataResult<fetcher_core::auth::Google> {
		data::google_oauth2::get(self.cx).into()
	}

	fn email_password(&self) -> ExternalDataResult<String> {
		data::email_password::get(self.cx).into()
	}

	fn telegram_bot_token(&self) -> ExternalDataResult<String> {
		data::telegram::get(self.cx).into()
	}

	fn read_filter(
		&self,
		name: &str,
		expected_rf: ReadFilterKind,
	) -> ExternalDataResult<ReadFilter> {
		read_filter::get(name, expected_rf, self.cx).into()
	}
}
