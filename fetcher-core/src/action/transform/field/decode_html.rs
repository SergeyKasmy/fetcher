/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`DecodeHtml`] field transform

use super::TransformField;
use crate::action::transform::result::TransformResult;
use std::convert::Infallible;

/// Decode HTML escape codes into their actual unicode forms, e.g. "&gt" -> ">"
#[derive(Debug)]
pub struct DecodeHtml;

impl TransformField for DecodeHtml {
	type Err = Infallible;

	fn transform_field(&self, old_val: Option<&str>) -> Result<TransformResult<String>, Self::Err> {
		let Some(old_val) = old_val else {
			return Ok(TransformResult::Old(None))
		};

		let escaped = html_escape::decode_html_entities(old_val).into_owned();

		Ok(TransformResult::New(Some(escaped)))
	}
}
