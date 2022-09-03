/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod filter;
pub mod transform;

use crate::entry::Entry;
use crate::error::transform::Error as TransformError;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // TODO: is there any benefit to this?
pub enum Action {
	Filter(filter::Kind),
	Transform(transform::Kind),
}

impl Action {
	pub async fn process(&self, mut entries: Vec<Entry>) -> Result<Vec<Entry>, TransformError> {
		match self {
			Action::Filter(f) => {
				f.filter(&mut entries).await;
				Ok(entries)
			}
			Action::Transform(tr) => {
				let mut fully_transformed = Vec::new();
				for entry in entries {
					fully_transformed.extend(tr.transform(entry).await?);
				}

				Ok(fully_transformed)
			}
		}
	}
}
