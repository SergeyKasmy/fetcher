/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use std::fs;

use super::TruncatingFileWriter;
use crate::settings::context::StaticContext;
use fetcher_config::jobs::{
	external_data::ExternalDataError, task::entry_to_msg_map::EntryToMsgMap as EntryToMsgMapConf,
	JobName, TaskId,
};
use fetcher_core::task::entry_to_msg_map::EntryToMsgMap;

const ENTRY_TO_MSG_MAP_DATA_DIR: &str = "entry_to_msg_map";

pub fn get(
	job: &JobName,
	task: Option<&TaskId>,
	cx: StaticContext,
) -> Result<EntryToMsgMap, ExternalDataError> {
	let path = {
		let mut path = cx.data_path.join(ENTRY_TO_MSG_MAP_DATA_DIR).join(&job.0);

		match task {
			Some(TaskId::Name(s)) => path.push(s),
			Some(TaskId::Id(i)) => path.push(i.to_string()),
			None => (),
		}

		path
	};

	match fs::read_to_string(&path) {
		Ok(map_raw) if map_raw.trim().is_empty() => {
			tracing::debug!("Entry to message map save file is empty");

			Ok(EntryToMsgMap::new(TruncatingFileWriter::new(path)))
		}
		Err(e) => {
			tracing::debug!("Read filter save file doesn't exist or is inaccessible: {e}");

			Ok(EntryToMsgMap::new(TruncatingFileWriter::new(path)))
		}
		Ok(map_raw) => {
			let conf: EntryToMsgMapConf = serde_json::from_str(&map_raw).map_err(|e| (e, &path))?;

			Ok(EntryToMsgMap::new_with_map(
				conf.parse(),
				TruncatingFileWriter::new(path),
			))
		}
	}
}
