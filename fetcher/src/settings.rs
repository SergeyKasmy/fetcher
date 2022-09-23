/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod config;
pub mod data;
pub mod read_filter;

use fetcher_config::tasks::external_data::{ExternalData, ExternalDataError};
use fetcher_core::read_filter::{Kind as ReadFilterKind, ReadFilter};

use once_cell::sync::OnceCell;
use std::path::PathBuf;

const PREFIX: &str = "fetcher";

pub static DATA_PATH: OnceCell<PathBuf> = OnceCell::new();
pub static CONF_PATHS: OnceCell<Vec<PathBuf>> = OnceCell::new();

struct ExternalDataFromDataDir {}

impl ExternalData for ExternalDataFromDataDir {
	fn twitter_token(&self) -> Result<Option<(String, String)>, ExternalDataError> {
		data::twitter::get()
	}

	fn google_oauth2(&self) -> Result<Option<fetcher_core::auth::Google>, ExternalDataError> {
		data::google_oauth2::get()
	}

	fn email_password(&self) -> Result<Option<String>, ExternalDataError> {
		data::email_password::get()
	}

	fn telegram_bot_token(&self) -> Result<Option<String>, ExternalDataError> {
		data::telegram::get()
	}

	fn read_filter(
		&self,
		name: &str,
		expected_rf: ReadFilterKind,
	) -> Result<ReadFilter, ExternalDataError> {
		read_filter::get(name, expected_rf)
	}
}
