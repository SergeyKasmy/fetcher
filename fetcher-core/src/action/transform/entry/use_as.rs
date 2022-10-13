/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the transform [`UseRawContents`]

use super::TransformEntry;
use crate::{
	action::transform::{
		field::Field,
		result::{TransformResult as TrRes, TransformedEntry},
	},
	entry::Entry,
	error::transform::Kind as TransformErrorKind,
	error::InvalidUrlError,
	utils::OptionExt,
};

use url::Url;

/// Use the value of a field as the value of a different field
#[derive(Debug)]
pub struct Use {
	/// use the value from this field
	pub field: Field,
	/// put the value into this field
	pub as_field: Field,
}

impl TransformEntry for Use {
	type Error = TransformErrorKind;

	fn transform_entry(&self, ent: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		let val = match self.field {
			Field::Title => ent.msg.title.clone(),
			Field::Body => ent.msg.body.clone(),
			Field::Link => ent.msg.link.as_ref().map(ToString::to_string),
			Field::RawContets => ent.raw_contents.clone(),
		};

		let mut ent = TransformedEntry::default();
		match self.as_field {
			Field::Title => ent.msg.title = TrRes::New(val),
			Field::Body => ent.msg.body = TrRes::New(val),
			Field::Link => {
				ent.msg.link = TrRes::New(val.try_map(|s| {
					Url::try_from(s.as_str()).map_err(|e| {
						TransformErrorKind::FieldLinkTransformInvalidUrl(InvalidUrlError(e, s))
					})
				})?);
			}

			Field::RawContets => ent.raw_contents = TrRes::New(val),
		}

		Ok(vec![ent])
	}
}
