/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod html;
pub mod json;

use serde::{Deserialize, Serialize};

use self::html::Html;
use self::json::Json;
use crate::error::ConfigError;
use fetcher_core::transform as core_transform;

#[allow(clippy::large_enum_variant)] // this enum is very short-lived, I don't think boxing is worth the trouble
#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub(crate) enum Transform {
	Http,
	Html(Html),
	Json(Json),
	Rss,

	ReadFilter,

	Caps,
}

impl Transform {
	pub(crate) fn parse(self) -> Result<core_transform::Transform, ConfigError> {
		Ok(match self {
			Transform::Http => core_transform::Transform::Http,
			Transform::Html(x) => core_transform::Transform::Html(x.parse()?),
			Transform::Json(x) => core_transform::Transform::Json(x.parse()),
			Transform::Rss => core_transform::Transform::Rss(core_transform::Rss {}),

			Transform::ReadFilter => unreachable!("If the transform was set to ReadFilter, it should've been parsed beforehand and it shouldn't be possible to reach here"),	// TODO: make this a compile-time guarantee probably
			Transform::Caps => core_transform::Transform::Caps(core_transform::Caps {}),
		})
	}
}
