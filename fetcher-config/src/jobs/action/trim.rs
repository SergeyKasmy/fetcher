/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::transform::{
	field::{trim::Trim as CTrim, TransformFieldWrapper as CTransformFieldWrapper},
	Transform as CTransform,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
#[serde(transparent)]
pub struct Trim {
	pub field: Field,
}

impl Trim {
	pub fn parse(self) -> impl CTransform {
		CTransformFieldWrapper {
			field: self.field.parse(),
			transformator: CTrim,
		}
	}
}
