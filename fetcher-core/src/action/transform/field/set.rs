/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Set`] field transform

use rand::seq::SliceRandom;
use std::convert::Infallible;

use super::TransformField;
use crate::action::transform::result::{OptionUnwrapTransformResultExt, TransformResult as TrRes};

/// Set a field to a hardcoded value
#[derive(Debug)]
pub struct Set(pub Option<Vec<String>>);

impl TransformField for Set {
	type Err = Infallible;

	fn transform_field(&self, _old_field: Option<&str>) -> Result<TrRes<String>, Self::Err> {
		Ok(self
			.0
			.as_ref()
			.and_then(|v| v.choose(&mut rand::thread_rng()))
			.cloned()
			.unwrap_or_empty())
	}
}
