/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::TransformField;
use crate::action::transform::result::TransformResult;

use std::convert::Infallible;
use std::iter::repeat;

#[derive(Debug)]
pub struct Shorten {
	pub len: usize,
}

impl TransformField for Shorten {
	type Error = Infallible;

	fn transform_field(&self, field: &str) -> Result<TransformResult<String>, Infallible> {
		let new_val = (self.len != 0).then(|| {
			field
				.chars()
				.take(self.len)
				.chain(repeat('.').take(3))
				.collect::<String>()
		});

		Ok(TransformResult::New(new_val))
	}
}