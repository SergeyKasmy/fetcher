/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::{
	Action as CAction,
	transform::field::{
		TransformFieldWrapper as CTransformFieldWrapper, shorten::Shorten as CShorten,
	},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Shorten(pub HashMap<Field, usize>);

impl Shorten {
	#[must_use]
	pub fn decode_from_conf(self) -> Vec<CAction> {
		self.0
			.into_iter()
			.map(|(field, len)| {
				CAction::Transform(Box::new(CTransformFieldWrapper {
					field: field.decode_from_conf(),
					transformator: CShorten { len },
				}))
			})
			.collect()
	}
}
