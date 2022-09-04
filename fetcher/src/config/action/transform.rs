/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod html;
pub mod json;
pub mod regex;
pub mod shorten;
pub mod trim;

use self::html::Html;
use self::json::Json;
use self::regex::Regex;
use self::shorten::Shorten;
use self::trim::Trim;
use crate::error::ConfigError;
use fetcher_core::action::transform as core_transform;
use fetcher_core::action::transform::Kind as CoreTransformKind;

use serde::{Deserialize, Serialize};

#[allow(clippy::large_enum_variant)] // this enum is very short-lived, I don't think boxing is worth the trouble
#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum Transform {
	Http,
	Html(Html),
	Json(Json),
	Feed,
	Regex(Regex),
	// Take(Take),
	UseRawContents,
	Caps,
	Trim(Trim),
	Shorten(Shorten),
	Print,
}

impl Transform {
	pub fn parse(self) -> Result<CoreTransformKind, ConfigError> {
		Ok(match self {
			Transform::Http => CoreTransformKind::Http,
			Transform::Html(x) => CoreTransformKind::Html(x.parse()?),
			Transform::Json(x) => CoreTransformKind::Json(x.parse()),
			Transform::Feed => CoreTransformKind::Feed(core_transform::Feed),
			Transform::Regex(x) => CoreTransformKind::Regex(x.parse()?),
			Transform::UseRawContents => {
				CoreTransformKind::UseRawContents(core_transform::UseRawContents)
			}
			Transform::Caps => CoreTransformKind::Caps(core_transform::Caps),
			Transform::Trim(x) => CoreTransformKind::Trim(x.parse()),
			Transform::Shorten(x) => CoreTransformKind::Shorten(x.parse()),
			Transform::Print => CoreTransformKind::Print,
		})
	}
}
