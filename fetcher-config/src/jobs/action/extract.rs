/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::Error as ConfigError;
use fetcher_core::action::transform::{
	field::{Extract as CExtract, TransformFieldWrapper as CTransformFieldWrapper},
	Transform as CTransform,
};

use serde::{Deserialize, Serialize};

use super::Field;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Extract {
	from_field: Field,
	re: String,
	passthrough_if_not_found: bool,
}

impl Extract {
	pub fn parse(self) -> Result<impl CTransform, ConfigError> {
		Ok(CTransformFieldWrapper {
			field: self.from_field.parse(),
			transformator: CExtract::new(&self.re, self.passthrough_if_not_found)?,
		})
	}
}
