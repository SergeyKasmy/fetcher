/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod contains;
pub mod extract;
pub mod html;
pub mod json;
pub mod remove_html;
pub mod replace;
pub mod set;
pub mod shorten;
pub mod take;
pub mod trim;
pub mod use_as;

use self::{
	contains::Contains, extract::Extract, html::Html, json::Json, remove_html::RemoveHtml,
	replace::Replace, set::Set, shorten::Shorten, take::Take, trim::Trim, use_as::Use,
};
use crate::Error;
use fetcher_core::{
	action::{
		transform::{
			field::{Field as CField, TransformFieldWrapper as CTransformFieldWrapper},
			Caps as CCaps, DebugPrint as CDebugPrint, Feed as CFeed, Http as CHttp,
		},
		Action as CAction,
	},
	read_filter::ReadFilter as CReadFilter,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
	// filters
	ReadFilter,
	Take(Take),
	Contains(Contains),

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
	Replace(Replace),
	Extract(Extract),
	RemoveHtml(RemoveHtml),
}

// TODO: add media
#[derive(Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Field {
	Title,
	Body,
	Link,
	Id,
	ReplyTo,
	RawContents,
}

impl Action {
	pub fn parse<RF>(self, rf: Option<RF>) -> Result<Option<Vec<CAction>>, Error>
	where
		RF: CReadFilter + 'static,
	{
		macro_rules! transform {
			($tr:expr) => {
				vec![CAction::Transform(Box::new($tr))]
			};
		}

		macro_rules! filter {
			($f:expr) => {
				vec![CAction::Filter(Box::new($f))]
			};
		}

		let act = match self {
			// filters
			Action::ReadFilter => {
				if let Some(rf) = rf {
					vec![CAction::Filter(Box::new(rf))]
				} else {
					tracing::warn!("Can't filter read entries when no read filter type is set up for the task!");
					return Ok(None);
				}
			}
			Action::Take(x) => filter!(x.parse()),
			Action::Contains(x) => filter!(x.parse()?),

			// entry transforms
			Action::Feed => transform!(CFeed),
			Action::Html(x) => transform!(x.parse()?),
			Action::Http => transform!(CHttp::new(CField::Link)?),
			Action::Json(x) => transform!(x.parse()?),
			Action::Use(x) => x.parse(),

			// field transforms
			Action::Caps => transform!(CTransformFieldWrapper {
				field: CField::Title,
				transformator: CCaps,
			}),
			Action::DebugPrint => transform!(CDebugPrint),
			Action::Set(s) => s.parse(),
			Action::Shorten(x) => x.parse(),
			Action::Trim(x) => transform!(x.parse()),
			Action::Replace(x) => transform!(x.parse()?),
			Action::Extract(x) => transform!(x.parse()?),
			Action::RemoveHtml(x) => x.parse()?,
		};

		Ok(Some(act))
	}
}

impl Field {
	pub fn parse(self) -> CField {
		match self {
			Field::Title => CField::Title,
			Field::Body => CField::Body,
			Field::Link => CField::Link,
			Field::Id => CField::Id,
			Field::ReplyTo => CField::ReplyTo,
			Field::RawContents => CField::RawContets,
		}
	}
}
