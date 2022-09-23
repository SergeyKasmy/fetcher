/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::action::Action;
use super::{external_data::ExternalData, read_filter, sink::Sink, source::Source};
use crate::tasks::ParsedTask;
use crate::Error;
use fetcher_core as fcore;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize, Debug)]
// TODO: add
// #[serde(deny_unknown_fields)]
// but allow templates templates field
// that's used elsewhere
pub struct Task {
	#[serde(rename = "read_filter_type")]
	read_filter_kind: Option<self::read_filter::Kind>,
	tag: Option<String>,
	refresh: u64,
	source: Source,
	#[serde(rename = "process")]
	actions: Option<Vec<Action>>,
	// TODO: several sinks
	sink: Option<Sink>,
}

impl Task {
	pub fn parse(self, name: &str, external: &dyn ExternalData) -> Result<ParsedTask, Error> {
		let rf = self
			.read_filter_kind
			.map(read_filter::Kind::parse)
			.map(|cfg_rf_kind| external.read_filter(name, cfg_rf_kind))
			.transpose()?
			.map(|rf| Arc::new(RwLock::new(rf)));

		let actions = self
			.actions
			.map(|x| {
				x.into_iter()
					.filter_map(|act| match act {
						Action::ReadFilter => match rf.clone() {
							Some(rf) => Some(Ok(fetcher_core::action::Action::Filter(
								fetcher_core::action::filter::Kind::ReadFilter(rf),
							))),
							None => {
								tracing::warn!("Can't use read filter transformer when no read filter is set up for the task!");
								None
							}
						},
						other => Some(other.parse()),
					})
					.collect::<Result<_, _>>()
			})
			.transpose()?;

		let inner = fcore::task::Task {
			tag: self.tag,
			source: self.source.parse(rf, external)?,
			actions,
			sink: self.sink.map(|x| x.parse(external)).transpose()?,
		};

		Ok(ParsedTask {
			inner,
			refresh: self.refresh,
		})
	}
}
