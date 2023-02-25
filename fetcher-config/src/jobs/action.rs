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
		macro_rules! transform {
			($tr:expr) => {
				CAction::Transform(Box::new($tr))
			};
		}

		macro_rules! filter {
			($f:expr) => {
				CAction::Filter(Box::new($f))
			};
		}

		Ok(match self {
			// filters
			Action::ReadFilter => unreachable!(),
			Action::Take(x) => filter!(x.parse()),

			// entry transforms
			Action::Feed => transform!(CFeed),
			Action::Html(x) => transform!(x.parse()?),
			Action::Http => transform!(CHttp::new(CField::Link)?),
			Action::Json(x) => transform!(x.parse()?),
			Action::Use(x) => transform!(x.parse()),

			// field transforms
			Action::Caps => transform!(CTransformFieldWrapper {
				field: CField::Title,
				transformator: CCaps,
			}),
			Action::DebugPrint => transform!(CDebugPrint),
			Action::Set(s) => transform!(s.parse()),
			Action::Shorten(x) => transform!(x.parse()),
			Action::Trim(x) => transform!(x.parse()),

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
