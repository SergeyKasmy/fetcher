/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::action::transform::{
	field::{
		shorten::Shorten as CShorten, Field as CField,
		TransformFieldWrapper as CTransformFieldWrapper,
	},
	Transform as CTransform,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Shorten {
	len: usize,
	// TODO: add
	// field: Field,
}

impl Shorten {
	pub fn parse(self) -> impl CTransform {
		CTransformFieldWrapper {
			// field: self.field.parse(),
			field: CField::Body,
			transformator: CShorten { len: self.len },
		}
	}
}
