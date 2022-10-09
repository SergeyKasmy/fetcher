/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the trait [`TransformField`] as well as all types that implement it
//! And [`Field`] enum that can be used to refer to a [`Message`](`crate::sink::Message`)'s field

pub mod caps;
pub mod set;
pub mod shorten;
pub mod trim;

use self::{caps::Caps, set::Set, shorten::Shorten, trim::Trim};
use super::result::TransformResult;
use crate::{
	action::regex::{action::Extract, action::Replace, Regex},
	error::transform::Kind as TransformErrorKind,
};

use derive_more::From;

/// A helper trait for transforms that transform a single field of an entry
pub trait TransformField {
	/// Error return type. May be [`Infallible`](`std::convert::Infallible`)
	type Error: Into<TransformErrorKind>;

	/// Transform the `field` into a new field or `None` specifying what happens if `None` is returned
	#[allow(clippy::missing_errors_doc)]
	fn transform_field(&self, field: Option<&str>) -> Result<TransformResult<String>, Self::Error>;
}

/// Type that includes all available transforms that implement the [`TransformField`] trait
#[allow(missing_docs)]
#[derive(From, Debug)]
pub enum Kind {
	RegexExtract(Regex<Extract>),
	RegexReplace(Regex<Replace>),
	Set(Set),
	Caps(Caps),
	Trim(Trim),
	Shorten(Shorten),
}

/// List of all available fields for transformations
#[derive(Debug)]
pub enum Field {
	/// The [`Message.title`] field
	Title,
	/// The [`Message.body`] field
	Body,
}

impl TransformField for Kind {
	type Error = TransformErrorKind;

	fn transform_field(
		&self,
		field: Option<&str>,
	) -> Result<TransformResult<String>, TransformErrorKind> {
		macro_rules! delegate {
		    ($($k:tt),+) => {
				match self {
					$(Self::$k(x) => x.transform_field(field).map_err(Into::into),)+
				}
		    };
		}

		delegate!(RegexExtract, RegexReplace, Set, Caps, Trim, Shorten)
	}
}
