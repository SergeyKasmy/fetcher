/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the transform [`Caps`] that's used mostly for debugging or testing purposes

use super::TransformField;
use crate::action::transform::result::TransformResult;
use crate::error::transform::Kind as TransformErrorKind;

/// Make all text in a field UPPERCASE
#[derive(Debug)]
pub struct Caps;

impl TransformField for Caps {
	// Infallible
	fn transform_field(
		&self,
		field: Option<&str>,
	) -> Result<TransformResult<String>, TransformErrorKind> {
		Ok(TransformResult::New(field.map(str::to_uppercase)))
	}
}
