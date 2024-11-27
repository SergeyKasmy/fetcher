/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::FetcherConfigError as ConfigError;
use fetcher_core::action::transform::{
	Transform as CTransform,
	field::{Replace as CReplace, TransformFieldWrapper as CTransformFieldWrapper},
};

use serde::{Deserialize, Serialize};

use super::Field;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Replace {
	pub re: String,
	pub in_field: Field,
	pub with: String,
}

impl Replace {
	pub fn decode_from_conf(self) -> Result<impl CTransform, ConfigError> {
		Ok(CTransformFieldWrapper {
			field: self.in_field.decode_from_conf(),
			transformator: CReplace::new(&self.re, self.with)?,
		})
	}
}
