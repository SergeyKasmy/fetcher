/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all [`Actions`](`Action`) that a list of [`Entry`]'s can be run through to view/modify/filter it out

pub mod filter;
pub mod transform;

use self::{filter::Filter, transform::Transform};

/// An action that modifies a list of entries in some way
#[derive(Debug)]
pub enum Action {
	/// Filter out entries
	Filter(Box<dyn Filter>),
	/// Transform some entries into one or more new entries
	Transform(Box<dyn Transform>),
}

impl From<Box<dyn Filter>> for Action {
	fn from(filter: Box<dyn Filter>) -> Self {
		Action::Filter(filter)
	}
}

impl From<Box<dyn Transform>> for Action {
	fn from(transform: Box<dyn Transform>) -> Self {
		Action::Transform(transform)
	}
}
