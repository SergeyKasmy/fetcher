/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use crate::Error;
use fetcher_core::action::{
	regex::{
		action::{Extract, Find, Replace},
		Regex as CRegex,
	},
	transform::field::Transform as CFieldTransform,
	Action as CAction,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Regex {
	pub re: String,
	pub action: Action,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
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
			Action::Find { in_field } => CAction::Filter(
				CRegex::new(
					re,
					Find {
						in_field: in_field.parse(),
					},
				)?
				.into(),
			),
			Action::Extract {
				from_field: field,
				passthrough_if_not_found,
			} => CFieldTransform {
				field: field.parse(),
				kind: CRegex::new(
					re,
					Extract {
						passthrough_if_not_found,
					},
				)?
				.into(),
			}
			.into(),
			Action::Replace { in_field, with } => CFieldTransform {
				field: in_field.parse(),
				kind: CRegex::new(re, Replace { with })?.into(),
			}
			.into(),
		})
	}
}
