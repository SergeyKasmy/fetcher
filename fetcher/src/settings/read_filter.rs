/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod external_save;

use self::external_save::TruncatingFileWriter;
use crate::settings::context::StaticContext as Context;
use fetcher_config::jobs::{
	external_data::ExternalDataError,
	read_filter::{Kind as ReadFilterKind, ReadFilter as ReadFilterConf},
};
use fetcher_core::read_filter::{self as core_rf, ReadFilter};

use std::fs;

const READ_DATA_DIR: &str = "read";

#[tracing::instrument]
pub fn get(
	name: &str,
	expected_rf_kind: ReadFilterKind,
	context: Context,
) -> Result<Box<dyn ReadFilter>, ExternalDataError> {
	let path = context.data_path.join(READ_DATA_DIR).join(name);

	match fs::read_to_string(&path) {
		Ok(save_file_rf_raw) if save_file_rf_raw.trim().is_empty() => {
			tracing::debug!("Read filter save file is empty");

			Ok(match expected_rf_kind {
				ReadFilterKind::NewerThanRead => Box::new(core_rf::Newer::new()),
				ReadFilterKind::NotPresentInReadList => Box::new(core_rf::NotPresent::new()),
			})
		}
		Err(e) => {
			tracing::debug!("Read filter save file doesn't exist or is inaccessible: {e}");

			Ok(match expected_rf_kind {
				ReadFilterKind::NewerThanRead => Box::new(core_rf::Newer::new()),
				ReadFilterKind::NotPresentInReadList => Box::new(core_rf::NotPresent::new()),
			})
		}
		Ok(save_file_rf_raw) => {
			let conf: ReadFilterConf =
				serde_json::from_str(&save_file_rf_raw).map_err(|e| (e, &path))?;

			// the old read filter saved on disk is of the same type as the one set in config
			if conf == expected_rf_kind {
				let rf = conf.parse(TruncatingFileWriter::new(path));
				Ok(rf)
			} else {
				Err(ExternalDataError::new_rf_incompat_with_path(
					expected_rf_kind,
					conf.to_kind(),
					&path,
				))
			}
		}
	}
}
