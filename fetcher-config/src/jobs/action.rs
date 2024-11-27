/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod contains;
pub mod decode_html;
pub mod extract;
pub mod html;
pub mod import;
pub mod json;
pub mod remove_html;
pub mod replace;
pub mod set;
pub mod shorten;
pub mod take;
pub mod trim;
pub mod use_as;

use self::{
	contains::Contains, decode_html::DecodeHtml, extract::Extract, html::Html, import::Import,
	json::Json, remove_html::RemoveHtml, replace::Replace, set::Set, shorten::Shorten, take::Take,
	trim::Trim, use_as::Use,
};
use super::{external_data::ProvideExternalData, sink::Sink};
use crate::FetcherConfigError;
use fetcher_core::{
	action::{
		Action as CAction,
		transform::{
			Caps as CCaps, DebugPrint as CDebugPrint, Feed as CFeed, Http as CHttp,
			field::{Field as CField, TransformFieldWrapper as CTransformFieldWrapper},
		},
	},
	read_filter::ReadFilter as CReadFilter,
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

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
	DecodeHtml(DecodeHtml),

	// other
	Sink(Sink),
	Import(Import),
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
	pub fn decode_from_conf<RF, D>(
		self,
		rf: Option<Arc<RwLock<RF>>>,
		external: &D,
	) -> Result<Option<Vec<CAction>>, FetcherConfigError>
	where
		RF: CReadFilter + 'static,
		D: ProvideExternalData + ?Sized,
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
					tracing::warn!(
						"Can't filter read entries when no read filter type is set up for the task!"
					);
					return Ok(None);
				}
			}
			Action::Take(x) => filter!(x.decode_from_conf()),
			Action::Contains(x) => x.decode_from_conf()?,

			// entry transforms
			Action::Feed => transform!(CFeed),
			Action::Html(x) => transform!(x.decode_from_conf()?),
			Action::Http => transform!(CHttp::new(CField::Link)?),
			Action::Json(x) => transform!(x.decode_from_conf()?),
			Action::Use(x) => x.decode_from_conf(),

			// field transforms
			Action::Caps => transform!(CTransformFieldWrapper {
				field: CField::Title,
				transformator: CCaps,
			}),
			Action::DebugPrint => transform!(CDebugPrint),
			Action::Set(s) => s.decode_from_conf(),
			Action::Shorten(x) => x.decode_from_conf(),
			Action::Trim(x) => transform!(x.decode_from_conf()),
			Action::Replace(x) => transform!(x.decode_from_conf()?),
			Action::Extract(x) => transform!(x.decode_from_conf()?),
			Action::RemoveHtml(x) => x.decode_from_conf()?,
			Action::DecodeHtml(x) => x.decode_from_conf(),

			// other
			Action::Sink(x) => vec![CAction::Sink(x.decode_from_conf(external)?)],
			Action::Import(x) => match x.decode_from_conf(rf, external) {
				Ok(Some(v)) => v,
				not_ok => return not_ok,
			},
		};

		Ok(Some(act))
	}
}

impl Field {
	#[must_use]
	pub fn decode_from_conf(self) -> CField {
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
