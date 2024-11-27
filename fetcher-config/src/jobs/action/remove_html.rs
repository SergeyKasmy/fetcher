/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use crate::FetcherConfigError as ConfigError;
use fetcher_core::action::{
	Action as CAction,
	transform::field::{
		Replace as CReplace, TransformFieldWrapper as CTransformFieldWrapper, Trim as CTrim,
		replace::HTML_TAG_RE,
	},
};

use itertools::process_results;
use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};

// Remove HTML tags and trim any remaining whitespace
#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RemoveHtml {
	#[serde_as(deserialize_as = "OneOrMany<_>")]
	pub r#in: Vec<Field>,
}

impl RemoveHtml {
	pub fn parse(self) -> Result<Vec<CAction>, ConfigError> {
		process_results(self.r#in.into_iter().map(remove_html_action_for), |i| {
			i.flatten().collect()
		})
	}
}

fn remove_html_action_for(field: Field) -> Result<[CAction; 2], ConfigError> {
	#[allow(clippy::manual_string_new)] // better shows the intent
	let remove_html = CReplace::new(HTML_TAG_RE, "".to_owned())?;

	let remove_html = CAction::Transform(Box::new(CTransformFieldWrapper {
		field: field.clone().parse(),
		transformator: remove_html,
	}));

	let trim = CAction::Transform(Box::new(CTransformFieldWrapper {
		field: field.parse(),
		transformator: CTrim,
	}));

	Ok([remove_html, trim])
}
