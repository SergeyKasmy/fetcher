/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::transform::field::trim::Trim as CTrim;
use fetcher_core::action::transform::field::Kind as CFieldTransformKind;
use fetcher_core::action::transform::field::Transform as CFieldTransform;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Trim {
	pub field: Field,
}

impl Trim {
	pub fn parse(self) -> CFieldTransform {
		CFieldTransform {
			field: self.field.parse(),
			kind: CFieldTransformKind::Trim(CTrim),
		}
	}
}
