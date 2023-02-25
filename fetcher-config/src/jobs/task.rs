/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{
	action::Action,
	external_data::{ExternalDataResult, ProvideExternalData},
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
	pub(crate) read_filter_kind: Option<read_filter::Kind>,
	pub(crate) tag: Option<String>,
	pub(crate) source: Option<Source>,
	#[serde(rename = "process")]
	pub(crate) actions: Option<Vec<Action>>,
	// TODO: several sinks or integrate into actions
	pub(crate) sink: Option<Sink>,
}

impl Task {
	pub fn parse<D>(self, name: &str, external: &D) -> Result<CTask, Error>
	where
		D: ProvideExternalData + ?Sized,
	{
		let rf = match self.read_filter_kind {
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
				.filter_map(|act| act.parse(rf.clone()).transpose())
				.collect::<Result<_, _>>()
		})?;

		Ok(CTask {
			tag: self.tag,
			source: self.source.map(|x| x.parse(rf, external)).transpose()?,
			actions,
			sink: self.sink.try_map(|x| x.parse(external))?,
		})
	}
}
