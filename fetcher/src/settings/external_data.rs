/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{context::StaticContext, data, read_filter};
use fetcher_config::jobs::{
	external_data::{ExternalDataResult, ProvideExternalData},
	read_filter::Kind as ReadFilterKind,
};
use fetcher_core::{auth, read_filter::ReadFilter, task::entry_to_msg_map::EntryToMsgMap};

pub struct ExternalDataFromDataDir {
	pub cx: StaticContext,
}

impl ProvideExternalData for ExternalDataFromDataDir {
	type ReadFilter = Box<dyn ReadFilter>;

	fn twitter_token(&self) -> ExternalDataResult<(String, String)> {
		data::twitter::get(self.cx).into()
	}

	fn google_oauth2(&self) -> ExternalDataResult<auth::Google> {
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
	) -> ExternalDataResult<Self::ReadFilter> {
		read_filter::get(name, expected_rf, self.cx).into()
	}

	fn entry_to_msg_map(&self, name: &str) -> ExternalDataResult<EntryToMsgMap> {
		data::entry_to_msg_map::get(name, self.cx).into()
	}
}
