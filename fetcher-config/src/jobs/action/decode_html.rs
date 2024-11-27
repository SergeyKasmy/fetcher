/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::{
	Action as CAction, transform::field::TransformFieldWrapper as CTransformFieldWrapper,
	transform::field::decode_html::DecodeHtml as CDecodeHtml,
};

use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};

// Decode HTML escape codes
#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DecodeHtml {
	#[serde_as(deserialize_as = "OneOrMany<_>")]
	pub r#in: Vec<Field>,
}

impl DecodeHtml {
	#[must_use]
	pub fn decode_from_conf(self) -> Vec<CAction> {
		self.r#in
			.into_iter()
			.map(|field| {
				CAction::Transform(Box::new(CTransformFieldWrapper {
					field: field.decode_from_conf(),
					transformator: CDecodeHtml,
				}))
			})
			.collect()
	}
}
