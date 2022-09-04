/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod config;
pub mod data;
pub mod read_filter;

use crate::{config::TaskSettings, error::ConfigError};

use std::future::Future;
use std::pin::Pin;

const PREFIX: &str = "fetcher";

pub async fn get_task_settings() -> Result<TaskSettings, ConfigError> {
	let read_filter_getter = |name: String, current| -> Pin<Box<dyn Future<Output = _>>> {
		Box::pin(async move { read_filter::get(&name, current).await })
	};

	Ok(TaskSettings {
		twitter_auth: data::twitter().await?,
		google_oauth2: data::google_oauth2().await?,
		email_password: data::email_password().await?,
		telegram: data::telegram().await?,
		read_filter: Box::new(read_filter_getter),
	})
}
