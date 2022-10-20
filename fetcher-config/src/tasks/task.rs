/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{action::Action, external_data::ExternalData, read_filter, sink::Sink, source::Source};
use crate::{tasks::ParsedTask, Error};
use fetcher_core as fcore;
use fetcher_core::utils::OptionExt;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type DisabledField = Option<bool>;
pub type TemplatesField = Option<Vec<String>>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Task {
	#[serde(rename = "read_filter_type")]
	read_filter_kind: Option<self::read_filter::Kind>,
	tag: Option<String>,
	refresh: u64,
	source: Source,
	#[serde(rename = "process")]
	actions: Option<Vec<Action>>,
	// TODO: several sinks or integrate into actions
	sink: Option<Sink>,

	// these are meant to be used externally and are unused here
	disabled: DisabledField,
	templates: TemplatesField,
}

impl Task {
	pub fn parse(self, name: &str, external: &dyn ExternalData) -> Result<ParsedTask, Error> {
		let rf = self
			.read_filter_kind
			.map(read_filter::Kind::parse)
			.try_map(|cfg_rf_kind| external.read_filter(name, cfg_rf_kind))?
			.map(|rf| Arc::new(RwLock::new(rf)));

		let actions = self.actions.try_map(|x| {
			x.into_iter()
				.filter_map(|act| match act {
					Action::ReadFilter => {
						if let Some(rf) = rf.clone() {
							Some(Ok(fetcher_core::action::Action::Filter(
								fetcher_core::action::filter::Kind::ReadFilter(rf),
							)))
						} else {
							tracing::warn!("Can't use read filter transformer when no read filter is set up for the task!");
							None
						}
					}
					other => Some(other.parse()),
				})
				.collect::<Result<_, _>>()
		})?;

		let inner = fcore::task::Task {
			tag: self.tag,
			source: self.source.parse(rf, external)?,
			actions,
			sink: self.sink.try_map(|x| x.parse(external))?,
		};

		Ok(ParsedTask {
			inner,
			refresh: self.refresh,
		})
	}
}
