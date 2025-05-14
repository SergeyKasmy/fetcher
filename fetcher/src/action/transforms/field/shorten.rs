/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Shorten`] transform

use super::TransformField;
use crate::{
	StaticStr,
	action::transforms::result::{OptionUnwrapTransformResultExt, TransformResult},
};

use std::{convert::Infallible, iter::repeat};

/// Shorten a field to [`len`](`Shorten::len`). Makes the field completely empty if [`len`](`Shorten::len`) is 0, or trims the field to [`len`](`Shorten::len`) and adds "..." to the end
#[derive(Debug)]
pub struct Shorten {
	/// The maximum length of the field string
	pub len: usize,
}

impl TransformField for Shorten {
	type Err = Infallible;

	fn transform_field(
		&self,
		field: Option<&str>,
	) -> Result<TransformResult<StaticStr>, Self::Err> {
		// len == 0 means we should unset the field. Same effect as Set with value: None here
		let new_val = if self.len == 0 {
			None
		} else if let Some(field) = field {
			// pass-through the field if it's shorter than max len
			if field.chars().count() < self.len {
				Some(field.to_owned())
			} else {
				// take self.len chars from field and append "..."
				Some(
					field
						.chars()
						.take(self.len)
						.chain(repeat('.').take(3))
						.collect::<String>(),
				)
			}
		} else {
			None
		};

		Ok(new_val.unwrap_or_empty())
	}
}
