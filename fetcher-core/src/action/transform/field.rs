/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod caps;
pub mod regex;
pub mod shorten;
pub mod trim;

use enum_dispatch::enum_dispatch;

use self::caps::Caps;
use self::regex::Regex;
use self::shorten::Shorten;
use self::trim::Trim;

#[enum_dispatch]
pub trait TransformField {
	fn transform_field(&self, field: &str) -> String;
}

#[enum_dispatch(TransformField)]
#[derive(Debug)]
pub enum Kind {
	Regex(Regex),
	Caps(Caps),
	Trim(Trim),
	Shorten(Shorten),
}

#[derive(Debug)]
pub struct Transform {
	pub field: Field,
	pub kind: Kind,
}

#[derive(Debug)]
pub enum Field {
	Title,
	Body,
}
