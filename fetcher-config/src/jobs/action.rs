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
		entry::Kind as CTransformEntryKind, field::caps::Caps as CCaps, field::Field as CField,
		field::Kind as CFieldTransformKind, Feed as CFeed, Transform as CTransform,
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
	Http,
	Html(Html),
	Json(Json),
	Feed,
	Use(Use),
	Print,

	// field transforms
	Set(Set),
	Caps,
	Trim(Trim),
	Shorten(Shorten),

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
			Action::ReadFilter => unreachable!(),
			Action::Take(x) => CAction::Filter(x.parse().into()),
			Action::Http => CTransformEntryKind::Http.into(),
			Action::Html(x) => x.parse()?.into(),
			Action::Json(x) => x.parse()?.into(),
			Action::Feed => CFeed.into(),
			Action::Use(x) => x.parse().into(),
			Action::Print => CTransformEntryKind::Print.into(),
			Action::Set(s) => s.parse().into(),
			Action::Caps => CAction::Transform(CTransform::Field {
				field: CField::Title,
				kind: CFieldTransformKind::Caps(CCaps),
			}),
			Action::Trim(x) => x.parse().into(),
			Action::Shorten(x) => x.parse().into(),
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