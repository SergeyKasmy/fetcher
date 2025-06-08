/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Trim`] field transform

use itertools::Itertools;
use std::convert::Infallible;

use super::TransformField;
use crate::actions::transforms::result::{OptionUnwrapTransformResultExt, TransformResult};

/// Trim whitespace from a field
#[derive(Debug)]
pub struct Trim;

impl TransformField for Trim {
	type Err = Infallible;

	async fn transform_field(
		&mut self,
		value: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		Ok(value.map(trim).unwrap_or_empty())
	}
}

/// Trims each line separately and joins them back
fn trim(s: &str) -> String {
	s.trim()
		.lines()
		.map(|line| line.trim().to_owned())
		.join("\n")
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
