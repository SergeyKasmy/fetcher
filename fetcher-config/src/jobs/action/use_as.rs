/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::{Action as CAction, transform::Use as CUse};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Use(pub HashMap<Field, As>);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct As {
	pub r#as: Field,
}

impl Use {
	#[must_use]
	pub fn decode_from_conf(self) -> Vec<CAction> {
		self.0
			.into_iter()
			.map(|(field, as_field)| {
				CAction::Transform(Box::new(CUse {
					field: field.decode_from_conf(),
					as_field: as_field.r#as.decode_from_conf(),
				}))
			})
			.collect()
	}
}
