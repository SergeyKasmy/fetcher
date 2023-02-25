/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use crate::Error;
use fetcher_core::action::{
	transform::field::{
		regex::{
			action::{Extract, Find, Replace},
			Regex as CRegex,
		},
		TransformFieldWrapper as CTransformFieldWrapper,
	},
	Action as CAction,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Regex {
	pub re: String,
	pub action: Action,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
	Find {
		in_field: Field,
	},
	Extract {
		from_field: Field,
		passthrough_if_not_found: bool,
	},
	Replace {
		in_field: Field,
		with: String,
	},
}

impl Regex {
	pub fn parse(self) -> Result<CAction, Error> {
		let re = &self.re;

		Ok(match self.action {
			Action::Find { in_field } => CAction::Filter(Box::new(CRegex::new(
				re,
				Find {
					in_field: in_field.parse(),
				},
			)?)),
			Action::Extract {
				from_field: field,
				passthrough_if_not_found,
			} => CAction::Transform(Box::new(CTransformFieldWrapper {
				field: field.parse(),
				transformator: CRegex::new(
					re,
					Extract {
						passthrough_if_not_found,
					},
				)?,
			})),
			Action::Replace { in_field, with } => {
				CAction::Transform(Box::new(CTransformFieldWrapper {
					field: in_field.parse(),
					transformator: CRegex::new(re, Replace { with })?,
				}))
			}
		})
	}
}
