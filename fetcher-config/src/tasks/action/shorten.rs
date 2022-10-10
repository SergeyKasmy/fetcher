/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::action::transform::field::shorten::Shorten as CShorten;
use fetcher_core::action::transform::field::Field as CField;
use fetcher_core::action::transform::field::Kind as CFieldTransformKind;
use fetcher_core::action::transform::Transform as CTransform;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Shorten {
	len: usize,
	// field: Field,
}

impl Shorten {
	pub fn parse(self) -> CTransform {
		CTransform::Field {
			// field: self.field.parse(),
			field: CField::Body,
			kind: CFieldTransformKind::Shorten(CShorten { len: self.len }),
		}
	}
}
