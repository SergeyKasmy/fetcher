/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod config;
pub mod data;
pub mod read_filter;

use fetcher_config::tasks::TaskSettings;
use fetcher_core::read_filter::Kind as ReadFilterKind;
use fetcher_core::read_filter::ReadFilter;

use once_cell::sync::OnceCell;
use std::io;
use std::path::PathBuf;

const PREFIX: &str = "fetcher";

pub static DATA_PATH: OnceCell<PathBuf> = OnceCell::new();
pub static CONF_PATHS: OnceCell<Vec<PathBuf>> = OnceCell::new();

struct TaskSettingsFromDataDir {}

impl TaskSettings for TaskSettingsFromDataDir {
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

	fn read_filter(&self, name: &str, expected_rf: ReadFilterKind) -> io::Result<ReadFilter> {
		read_filter::get(name, expected_rf)
	}
}
