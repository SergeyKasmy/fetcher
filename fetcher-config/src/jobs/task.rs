/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod entry_to_msg_map;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tap::TapOptional;
use tokio::sync::RwLock;

use super::{
	action::Action,
	external_data::{ExternalDataResult, ProvideExternalData},
	named::{JobName, TaskName},
	read_filter,
	sink::Sink,
	source::Source,
};
use crate::Error;
use fetcher_core::{task::Task as CTask, utils::OptionExt};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Task {
	#[serde(rename = "read_filter_type")]
	pub read_filter_kind: Option<read_filter::Kind>,
	pub tag: Option<String>,
	pub source: Option<Source>,
	#[serde(rename = "process")]
	pub actions: Option<Vec<Action>>,
	// TODO: completely integrate into actions
	pub sink: Option<Sink>,
	pub entry_to_msg_map_enabled: Option<bool>,
}

impl Task {
	#[tracing::instrument(level = "debug", skip(self, external))]
	pub fn parse<D>(
		self,
		job: &JobName,
		task_name: Option<&TaskName>,
		external: &D,
	) -> Result<CTask, Error>
	where
		D: ProvideExternalData + ?Sized,
	{
		tracing::trace!("Parsing task config");

		let rf = match self.read_filter_kind {
			Some(expected_rf_type) => {
				match external.read_filter(job, task_name, expected_rf_type) {
					ExternalDataResult::Ok(rf) => Some(Arc::new(RwLock::new(rf))),
					ExternalDataResult::Unavailable => {
						tracing::info!("Read filter is unavailable, skipping");
						None
					}
					ExternalDataResult::Err(e) => return Err(e.into()),
				}
			}
			None => None,
		};

		let actions = self.actions.try_map(|acts| {
			itertools::process_results(
				acts.into_iter()
					.filter_map(|act| act.parse(rf.clone()).transpose()),
				|i| i.flatten().collect(),
			)
		})?;

		// TODO: replace with match like tag below
		let entry_to_msg_map = if self
			.entry_to_msg_map_enabled
			.tap_some(|b| {
				if let Some(sink) = &self.sink {
					// TODO: include task name
					tracing::info!(
						"Overriding entry_to_msg_map_enabled for {} from the default {} to {}",
						job,
						sink.has_message_id_support(),
						b
					);
				}
			})
			.unwrap_or_else(|| {
				// TODO: replace with "source.supports_replies()". There's a point to keeping the map even if the sink doesn't support it, e.g. if it's changed from stdout to discord later on
				self.sink
					.as_ref()
					.map_or(false, Sink::has_message_id_support)
			}) {
			match external.entry_to_msg_map(job, task_name) {
				ExternalDataResult::Ok(v) => Some(v),
				ExternalDataResult::Unavailable => {
					tracing::info!("Entry to message map is unavailable, skipping...");
					None
				}
				ExternalDataResult::Err(e) => return Err(e.into()),
			}
		} else {
			None
		};

		let tag = match (self.tag, task_name) {
			(Some(tag_override), Some(task_name)) => {
				tracing::debug!(
					"Overriding tag from task name {task_name:?} with {tag_override:?}"
				);
				Some(tag_override)
			}
			(Some(tag), None) => {
				tracing::debug!("Setting custom tag {tag:?}");
				Some(tag)
			}
			(None, Some(task_name)) => {
				tracing::trace!("Using task name as tag");
				Some(task_name.as_str().to_string())
			}
			(None, None) => None,
		};

		Ok(CTask {
			tag,
			source: self.source.map(|x| x.parse(rf, external)).transpose()?,
			actions,
			sink: self.sink.try_map(|x| x.parse(external))?,
			entry_to_msg_map,
		})
	}
}
