/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Trim`] field transform

use itertools::Itertools;
use std::convert::Infallible;

use super::TransformField;
use crate::action::transform::result::TransformResult;

/// Trim whitespace from a field
#[derive(Debug)]
pub struct Trim;

impl TransformField for Trim {
	type Err = Infallible;

	fn transform_field(&self, old_val: Option<&str>) -> Result<TransformResult<String>, Self::Err> {
		Ok(TransformResult::New(old_val.map(trim)))
	}
}

fn trim(s: &str) -> String {
	s.trim().lines().map(|x| x.trim().to_owned()).join("\n")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn one_line() {
		const S: &str = "\n\n\n   \nHello, World!      \n    \n";
		assert_eq!(trim(S), "Hello, World!");
	}

	#[test]
	fn multi_line() {
		const S: &str = "\n\n\n   \nHello, \n   World!      \n    \n";
		assert_eq!(trim(S), "Hello,\nWorld!");
	}
}
