/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::TransformField;
use crate::action::transform::result::TransformResult;

#[derive(Debug)]
pub struct Caps;

impl TransformField for Caps {
	fn transform_field(&self, field: &str) -> TransformResult<String> {
		TransformResult::New(Some(field.to_uppercase()))
	}
}
