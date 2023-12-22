/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::FetcherConfigError as ConfigError;
use fetcher_core::action::transform::{
	field::{Replace as CReplace, TransformFieldWrapper as CTransformFieldWrapper},
	Transform as CTransform,
};

use serde::{Deserialize, Serialize};

use super::Field;

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Replace {
	pub re: String,
	pub in_field: Field,
	pub with: String,
}

impl Replace {
	pub fn parse(self) -> Result<impl CTransform, ConfigError> {
		Ok(CTransformFieldWrapper {
			field: self.in_field.parse(),
			transformator: CReplace::new(&self.re, self.with)?,
		})
	}
}
