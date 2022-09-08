/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod action;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;

pub use self::task::Task;

use fetcher_core as fcore;
use fetcher_core::read_filter::Kind as ReadFilterKind;
use fetcher_core::read_filter::ReadFilter;
use fetcher_core::task::Task as CoreTask;

use std::collections::HashMap;
use std::io;

pub type ParsedTasks = HashMap<String, ParsedTask>;
pub struct ParsedTask {
	pub inner: CoreTask,
	pub refresh: u64,
}

pub trait TaskSettings {
	fn twitter_token(&self) -> io::Result<Option<(String, String)>>;
	fn google_oauth2(&self) -> io::Result<Option<fcore::auth::Google>>;
	fn email_password(&self) -> io::Result<Option<String>>;
	fn telegram_bot_token(&self) -> io::Result<Option<String>>;
	fn read_filter(&self, name: &str, expected_rf: ReadFilterKind) -> io::Result<ReadFilter>;
}
