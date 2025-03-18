/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the transform [`Use`] that allows using the content of a [`Field`] as the new value of a different [`Field`]

use super::Transform;
use crate::{
	action::transforms::{
		error::TransformErrorKind,
		field::Field,
		result::{OptionUnwrapTransformResultExt, TransformedEntry},
	},
	entry::Entry,
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

impl Transform for Use {
	type Err = TransformErrorKind;

	async fn transform_entry(&self, ent: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let val = match self.field {
			Field::Title => ent.msg.title,
			Field::Body => ent.msg.body,
			Field::Link => ent.msg.link.map(|s| s.to_string()),
			Field::Id => ent.id.map(|id| id.0),
			Field::ReplyTo => ent.reply_to.map(|id| id.0),
			Field::RawContets => ent.raw_contents,
		};

		let mut ent = TransformedEntry::default();
		match self.as_field {
			Field::Title => ent.msg.title = val.unwrap_or_empty(),
			Field::Body => ent.msg.body = val.unwrap_or_empty(),
			Field::Link => {
				ent.msg.link = val
					.try_map(|s| {
						Url::try_from(s.as_str()).map_err(|e| {
							TransformErrorKind::FieldLinkTransformInvalidUrl(InvalidUrlError(e, s))
						})
					})?
					.unwrap_or_empty();
			}
			Field::Id => ent.id = val.map(Into::into).unwrap_or_empty(),
			Field::ReplyTo => ent.reply_to = val.map(Into::into).unwrap_or_empty(),
			Field::RawContets => ent.raw_contents = val.unwrap_or_empty(),
		}

		Ok(vec![ent])
	}
}
