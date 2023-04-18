/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::{
	transform::field::{
		shorten::Shorten as CShorten, TransformFieldWrapper as CTransformFieldWrapper,
	},
	Action as CAction,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
#[serde(transparent)]
pub struct Shorten(pub HashMap<Field, usize>);

impl Shorten {
	pub fn parse(self) -> Vec<CAction> {
		self.0
			.into_iter()
			.map(|(field, len)| {
				CAction::Transform(Box::new(CTransformFieldWrapper {
					field: field.parse(),
					transformator: CShorten { len },
				}))
			})
			.collect()
	}
}
