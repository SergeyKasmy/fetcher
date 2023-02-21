/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all [`Actions`](`Action`) that a list of [`Entry`]'s can be run through to view/modify/filter it out

pub mod filter;
pub mod transform;

// Regex is both a transform and a filter that's why it's here all alone :(
mod regex;

use self::{filter::Filter, transform::Transform};
use crate::{entry::Entry, error::transform::Error as TransformError};

/// An action that modifies a list of entries in some way
#[derive(Debug)]
pub enum Action {
	/// Filter out entries
	Filter(Box<dyn Filter>),
	/// Transform some entries into one or more new entries
	Transform(Box<dyn Transform>),
}

impl Action {
	/// Processes `entries` using the [`Action`]
	///
	/// # Errors
	/// if there was error transforming `entries`. Filtering out never fails
	pub async fn process(&self, mut entries: Vec<Entry>) -> Result<Vec<Entry>, TransformError> {
		match self {
			Action::Filter(f) => {
				f.filter(&mut entries).await;
				Ok(entries)
			}
			Action::Transform(tr) => {
				let mut fully_transformed = Vec::new();

				for entry in entries {
					fully_transformed.extend(tr.transform(&entry).await?);
				}

				Ok(fully_transformed)
			}
		}
	}
}

/*
// can't make this generic because of conflicting impl with the Into<Transform> one :(
impl From<filter::Kind> for Action {
	fn from(filter: filter::Kind) -> Self {
		Action::Filter(filter)
	}
}

impl<T: Into<Transform>> From<T> for Action {
	fn from(transform: T) -> Self {
		Action::Transform(transform.into())
	}
}
*/
