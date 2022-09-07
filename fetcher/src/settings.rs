/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod config;
pub mod data;
pub mod read_filter;

use fetcher_config::tasks::TaskSettings;

use std::io;

const PREFIX: &str = "fetcher";

struct TaskSettingsFetcherDefault;

impl TaskSettings for TaskSettingsFetcherDefault {
	fn twitter_token(&self) -> io::Result<Option<(String, String)>> {
		data::twitter::get()
	}

	fn google_oauth2(&self) -> io::Result<Option<fetcher_core::auth::Google>> {
		data::google_oauth2::get()
	}

	fn email_password(&self) -> io::Result<Option<String>> {
		data::email_password::get()
	}

	fn telegram_bot_token(&self) -> io::Result<Option<String>> {
		data::telegram::get()
	}

	fn read_filter(
		&self,
		name: &str,
		kind: fetcher_core::read_filter::Kind,
	) -> io::Result<fetcher_core::read_filter::ReadFilter> {
		todo!()
	}
}
