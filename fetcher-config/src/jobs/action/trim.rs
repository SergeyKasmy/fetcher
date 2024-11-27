/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::transform::{
	Transform as CTransform,
	field::{TransformFieldWrapper as CTransformFieldWrapper, trim::Trim as CTrim},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Trim {
	pub field: Field,
}

impl Trim {
	#[must_use]
	pub fn decode_from_conf(self) -> impl CTransform {
		CTransformFieldWrapper {
			field: self.field.decode_from_conf(),
			transformator: CTrim,
		}
	}
}
