/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::transform::{
	field::set::Set as CSet, field::TransformFieldWrapper as CTransformFieldWrapper,
	Transform as CTransform,
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Set {
	pub field: Field,
	pub value: Option<Values>,
}

impl Set {
	pub fn parse(self) -> Box<dyn CTransform> {
		Box::new(CTransformFieldWrapper {
			field: self.field.parse(),
			transformator: Box::new(CSet(self.value.map(|x| x.0))),
		})
	}
}

#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Values(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<String>);
