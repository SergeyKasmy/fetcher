/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add deny_unknown_fields annotations to every config struct
// TODO: mb rename .parse() into .into() or something of that sort? .into() is already used by From/Into traits though. Naming is hard, man... UPD: into_conf() and from_conf() are way better!

pub mod auth;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;
pub mod transform;

use std::future::Future;
use std::pin::Pin;

use serde::Deserialize;
use serde::Serialize;

pub use self::task::Task;
pub use self::task::TemplatesField;
use crate::error::ConfigError;
use fetcher_core::read_filter::ReadFilter;

pub(crate) type ReadFilterGetter = Box<
	dyn Fn(
		String,
		Option<fetcher_core::read_filter::Kind>,
	) -> Pin<Box<dyn Future<Output = Result<Option<ReadFilter>, ConfigError>>>>,
>;

pub(crate) struct DataSettings {
	pub twitter_auth: Option<(String, String)>,
	pub google_oauth2: Option<fetcher_core::auth::Google>,
	pub email_password: Option<String>,
	pub telegram: Option<teloxide::Bot>,
	pub read_filter: ReadFilterGetter,
}

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
