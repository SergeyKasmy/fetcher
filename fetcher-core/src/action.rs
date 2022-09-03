/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod transform;

use crate::entry::Entry;
use crate::error::transform::Error as TransformError;

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum Action {
	Filter,
	Transform(transform::Kind),
}

impl Action {
	pub async fn process(&self, mut entries: Vec<Entry>) -> Result<Vec<Entry>, TransformError> {
		match self {
			Action::Filter => todo!(),
			Action::Transform(tr) => {
				let mut fully_transformed_entries = Vec::new();
				for entry in entries {
					fully_transformed_entries.extend(tr.transform(entry).await?);
				}

				Ok(fully_transformed_entries)
			}
		}
	}
}
