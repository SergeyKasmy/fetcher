/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add deny_unknown_fields annotations to every config struct
// TODO: mb rename .parse() into .into() or something of that sort? .into() is already used by From/Into traits though. Naming is hard, man... UPD: into_conf() and from_conf() are way better!

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)] // TODO
// #![warn(missing_docs)]
#![warn(clippy::unwrap_used)]

pub mod error;
pub mod settings;
pub mod tasks;

pub use self::error::Error;

use serde::Deserialize;
use serde::Serialize;

// TODO: either rename to OneOrSeveral or use serde_with's alternative instead
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum OneOrMultiple<T> {
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
