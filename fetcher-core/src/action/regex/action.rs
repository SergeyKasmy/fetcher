/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::action::transform::field::Field;

#[derive(Debug)]
pub struct Extract {
	pub passthrough_if_not_found: bool,
}

#[derive(Debug)]
pub struct Find {
	pub in_field: Field,
}

#[derive(Debug)]
pub struct Replace {
	pub with: String,
}

pub trait Action {}
impl Action for Extract {}
impl Action for Find {}
impl Action for Replace {}
