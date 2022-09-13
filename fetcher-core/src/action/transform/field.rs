/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod caps;
pub mod shorten;
pub mod trim;

use self::caps::Caps;
use self::shorten::Shorten;
use self::trim::Trim;
use super::result::TransformResult;
use crate::{
	action::regex::{action::Extract, action::Replace, Regex},
	error::transform::Kind as TransformErrorKind,
};

use derive_more::From;

pub trait TransformField {
	type Error: Into<TransformErrorKind>;

	fn transform_field(&self, field: &str) -> Result<TransformResult<String>, Self::Error>;
}

#[derive(Debug)]
pub struct Transform {
	pub field: Field,
	pub kind: Kind,
}

#[derive(From, Debug)]
pub enum Kind {
	RegexExtract(Regex<Extract>),
	RegexReplace(Regex<Replace>),
	Caps(Caps),
	Trim(Trim),
	Shorten(Shorten),
}

#[derive(Debug)]
pub enum Field {
	Title,
	Body,
}

impl Kind {
	pub fn transform_field(
		&self,
		field: &str,
	) -> Result<TransformResult<String>, TransformErrorKind> {
		macro_rules! delegate {
		    ($($k:tt),+) => {
				match self {
					$(Self::$k(x) => x.transform_field(field).map_err(Into::into),)+
				}
		    };
		}

		delegate!(RegexExtract, RegexReplace, Caps, Trim, Shorten)
	}
}
