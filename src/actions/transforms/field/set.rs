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
	actions::transforms::result::{OptionUnwrapTransformResultExt, TransformResult as TrRes},
};

/// Set a field to a hardcoded string, a random string, or nothing at all
#[derive(Debug)]
pub enum Set {
	/// One single always the same hardcoded string
	Single(StaticStr),

	/// A random string from a list of hardcoded strings
	Random(Vec<StaticStr>),

	/// Empty the field. In other words: make it [`None`]
	Empty,
}

impl Set {
	/// Creates a new [`Set::single`] with the provided string
	pub fn single<S>(string: S) -> Self
	where
		S: Into<StaticStr>,
	{
		Self::Single(string.into())
	}

	/// Creates a new [`Set::random`] with the provided strings
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

	fn transform_field(&self, _old_field: Option<&str>) -> Result<TrRes<String>, Self::Err> {
		Ok(match self {
			Set::Single(x) => TrRes::New(x.to_string()),
			Set::Random(vec) => vec
				.choose(&mut rand::thread_rng())
				.map(ToString::to_string)
				.unwrap_or_empty(),
			Set::Empty => TrRes::Empty,
		})
	}
}
