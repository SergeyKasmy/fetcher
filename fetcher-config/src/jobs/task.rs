/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::Error;

use super::{
	action::Action,
	external_data::{ExternalDataResult, ProvideExternalData},
	read_filter,
	sink::Sink,
	source::Source,
};
use fetcher_core::{task::Task as CoreTask, utils::OptionExt};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Task {
	#[serde(rename = "read_filter_type")]
	pub(crate) read_filter_kind: Option<read_filter::Kind>,
	pub(crate) tag: Option<String>,
	pub(crate) source: Source,
	#[serde(rename = "process")]
	pub(crate) actions: Option<Vec<Action>>,
	// TODO: several sinks or integrate into actions
	pub(crate) sink: Option<Sink>,
}

impl Task {
	pub fn parse(self, name: &str, external: &dyn ProvideExternalData) -> Result<CoreTask, Error> {
		let rf = match self.read_filter_kind.map(read_filter::Kind::parse) {
			Some(expected_rf_type) => match external.read_filter(name, expected_rf_type) {
				ExternalDataResult::Ok(rf) => Some(Arc::new(RwLock::new(rf))),
				ExternalDataResult::Unavailable => {
					tracing::warn!("Read filter is unavailable, skipping");
					None
				}
				ExternalDataResult::Err(e) => return Err(e.into()),
			},
			None => None,
		};

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

		Ok(CoreTask {
			tag: self.tag,
			source: self.source.parse(rf, external)?,
			actions,
			sink: self.sink.try_map(|x| x.parse(external))?,
		})
	}
}
