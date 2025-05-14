/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Set`] field transform

use rand::seq::SliceRandom;
use std::convert::Infallible;

use super::TransformField;
use crate::{
	StaticStr,
	action::transforms::result::{OptionUnwrapTransformResultExt, TransformResult as TrRes},
};

/// Set a field to a hardcoded value
#[derive(Debug)]
pub enum Set {
	Single(StaticStr),
	Random(Vec<StaticStr>),
	Empty,
}

impl Set {
	pub fn single<S>(string: S) -> Self
	where
		S: Into<StaticStr>,
	{
		Self::Single(string.into())
	}

	pub fn random<I, S>(iter: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<StaticStr>,
	{
		Self::Random(iter.into_iter().map(Into::into).collect())
	}
}

impl TransformField for Set {
	type Err = Infallible;

	// FIXME: remove .to_string(), change TransformField to at least return a StaticStr, hopefully a totally generic type
	fn transform_field(&self, _old_field: Option<&str>) -> Result<TrRes<StaticStr>, Self::Err> {
		Ok(match self {
			Set::Single(x) => TrRes::New(x.clone()),
			Set::Random(vec) => vec
				.choose(&mut rand::thread_rng())
				.cloned()
				.unwrap_or_empty(),
			Set::Empty => TrRes::Empty,
		})
	}
}
