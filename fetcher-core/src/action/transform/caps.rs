/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::TransformField;

#[derive(Debug)]
pub struct Caps;

impl TransformField for Caps {
	fn transform_field(&self, field: &str) -> String {
		field.to_uppercase()
	}
}
