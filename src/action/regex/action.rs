/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all actions that [`Regex`](`super::Regex`) can be used for

use crate::action::transform::field::Field;

#[allow(rustdoc::invalid_html_tags)]
/// Extract some text via regex, contained in capture group "s" (?P<s>)
#[derive(Debug)]
pub struct Extract {
	/// Passthrough previous text if the capture group wasn't found
	pub passthrough_if_not_found: bool,
}

/// Find a re in a field
#[derive(Debug)]
pub struct Find {
	/// The field to find the re in
	pub in_field: Field,
}

/// Replace text that matched the re with a replacement string
#[derive(Debug)]
pub struct Replace {
	/// The text to replace with. May reference capture groups in the re
	pub with: String,
}

/// A stub trait that all regex actions implement
pub trait Action: sealed::Sealed {}
impl Action for Extract {}
impl Action for Find {}
impl Action for Replace {}

mod sealed {
	use super::{Extract, Find, Replace};

	pub trait Sealed {}
	impl Sealed for Extract {}
	impl Sealed for Find {}
	impl Sealed for Replace {}
}
