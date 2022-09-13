/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::action::transform::field::shorten::Shorten as CShorten;
use fetcher_core::action::transform::field::Field as CField;
use fetcher_core::action::transform::field::Kind as CFieldTransformKind;
use fetcher_core::action::transform::field::Transform as CFieldTransform;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Shorten {
	// field: Field,
	len: usize,
}

impl Shorten {
	pub fn parse(self) -> CFieldTransform {
		CFieldTransform {
			// field: self.field.parse(),
			field: CField::Body,
			kind: CFieldTransformKind::Shorten(CShorten { len: self.len }),
		}
	}
}
