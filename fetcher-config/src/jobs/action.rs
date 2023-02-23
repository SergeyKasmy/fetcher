/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod html;
pub mod json;
pub mod regex;
pub mod set;
pub mod shorten;
pub mod take;
pub mod trim;
pub mod use_as;

use self::{
	html::Html, json::Json, regex::Regex, set::Set, shorten::Shorten, take::Take, trim::Trim,
	use_as::Use,
};
use crate::Error;
use fetcher_core::action::{
	transform::{
		field::{Field as CField, TransformFieldWrapper as CTransformFieldWrapper},
		Caps as CCaps, DebugPrint as CDebugPrint, Feed as CFeed, Http as CHttp,
	},
	Action as CAction,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
	// filters
	ReadFilter,
	Take(Take),

	// entry transforms
	DebugPrint,
	Feed,
	Html(Html),
	Http,
	Json(Json),
	Use(Use),

	// field transforms
	Caps,
	Set(Set),
	Shorten(Shorten),
	Trim(Trim),

	// other
	Regex(Regex),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Field {
	Title,
	Body,
	Link,
	RawContents,
}

impl Action {
	pub fn parse(self) -> Result<CAction, Error> {
		Ok(match self {
			// filters
			Action::ReadFilter => unreachable!(),
			Action::Take(x) => CAction::Filter(x.parse()),

			// entry transforms
			Action::Feed => CAction::Transform(Box::new(CFeed)),
			Action::Html(x) => x.parse()?.into(),
			Action::Http => CAction::Transform(Box::new(CHttp::new(CField::Link)?)),
			Action::Json(x) => x.parse()?.into(),
			Action::Use(x) => x.parse().into(),

			// field transforms
			Action::Caps => CAction::Transform(Box::new(CTransformFieldWrapper {
				field: CField::Title,
				transformator: Box::new(CCaps),
			})),
			Action::DebugPrint => CAction::Transform(Box::new(CDebugPrint)),
			Action::Set(s) => s.parse().into(),
			Action::Shorten(x) => x.parse().into(),
			Action::Trim(x) => x.parse().into(),

			// other
			Action::Regex(x) => x.parse()?,
		})
	}
}

impl Field {
	pub fn parse(self) -> CField {
		match self {
			Field::Title => CField::Title,
			Field::Body => CField::Body,
			Field::Link => CField::Link,
			Field::RawContents => CField::RawContets,
		}
	}
}
