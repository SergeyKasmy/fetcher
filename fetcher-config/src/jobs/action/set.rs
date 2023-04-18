/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;

use super::Field;
use fetcher_core::action::transform::{
	field::set::Set as CSet, field::TransformFieldWrapper as CTransformFieldWrapper,
};
use fetcher_core::action::Action as CAction;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
#[serde(transparent)]
pub struct Set(pub HashMap<Field, Option<Values>>);

impl Set {
	pub fn parse(self) -> Vec<CAction> {
		self.0
			.into_iter()
			.map(|(field, values)| {
				CAction::Transform(Box::new(CTransformFieldWrapper {
					field: field.parse(),
					transformator: CSet(values.map(|x| x.0)),
				}))
			})
			.collect()
	}
}

#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Values(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<String>);
