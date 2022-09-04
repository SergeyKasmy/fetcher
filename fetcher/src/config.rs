/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add deny_unknown_fields annotations to every config struct
// TODO: mb rename .parse() into .into() or something of that sort? .into() is already used by From/Into traits though. Naming is hard, man... UPD: into_conf() and from_conf() are way better!

pub mod action;
pub mod auth;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;

use std::future::Future;
use std::pin::Pin;

use serde::Deserialize;
use serde::Serialize;

pub use self::task::Task;
use crate::error::ConfigError;
use fetcher_core::read_filter::ReadFilter;

/// A struct to pass around in the config module in order not to depend on the settings module directly
/// All settings are shared except for the read_filter which is separate for each task and requires a name and a default value to get
// This one should be especially useful if we decide to move the config module out into a separate crate
pub struct TaskSettings {
	pub twitter_auth: Option<(String, String)>,
	pub google_oauth2: Option<fetcher_core::auth::Google>,
	pub email_password: Option<String>,
	pub telegram: Option<String>,
	pub read_filter: ReadFilterGetter,
}

/// A closure that takes the name of the task and its default read filter kind and returns a future
/// that returns a result if there was an error parsing current config/save file and an option if the default value is none
pub type ReadFilterGetter = Box<
	dyn Fn(
		String,
		Option<fetcher_core::read_filter::Kind>,
	) -> Pin<Box<dyn Future<Output = Result<Option<ReadFilter>, ConfigError>>>>,
>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum OneOrMultiple<T> {
	One(T),
	Multiple(Vec<T>),
}

impl<T> From<OneOrMultiple<T>> for Vec<T> {
	fn from(one_or_mltp: OneOrMultiple<T>) -> Self {
		match one_or_mltp {
			OneOrMultiple::One(x) => vec![x],
			OneOrMultiple::Multiple(x) => x,
		}
	}
}
