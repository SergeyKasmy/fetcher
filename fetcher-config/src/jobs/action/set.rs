/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::transform::{
	field::{set::Set as CSet, Kind as CFieldTransformKind},
	Transform as CTransform,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Set {
	pub field: Field,
	pub value: Option<String>,
}

impl Set {
	pub fn parse(self) -> CTransform {
		CTransform::Field {
			field: self.field.parse(),
			kind: CFieldTransformKind::Set(CSet(self.value)),
		}
	}
}
