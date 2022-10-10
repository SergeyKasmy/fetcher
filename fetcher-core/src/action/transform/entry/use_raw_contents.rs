/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the transform [`UseRawContents`]

use super::TransformEntry;
use crate::action::transform::result::{
	TransformResult as TrRes, TransformedEntry, TransformedMessage,
};
use crate::entry::Entry;

use std::convert::Infallible;

/// Use the [`Entry.raw_contents`] field as [`Message.body`]
#[derive(Debug)]
pub struct UseRawContents;

impl TransformEntry for UseRawContents {
	type Error = Infallible;

	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		Ok(vec![transform_impl(entry)])
	}
}

fn transform_impl(entry: &Entry) -> TransformedEntry {
	TransformedEntry {
		msg: TransformedMessage {
			body: TrRes::New(entry.raw_contents.clone()),
			..Default::default()
		},
		..Default::default()
	}
}
