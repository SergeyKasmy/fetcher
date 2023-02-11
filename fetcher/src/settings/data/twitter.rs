/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::prompt_user_for;
use crate::settings::context::StaticContext as Context;
use fetcher_config::{settings::Twitter as Config, tasks::external_data::ExternalDataError};

use std::fs;

const FILE_NAME: &str = "twitter.json";

pub fn get(cx: Context) -> Result<(String, String), ExternalDataError> {
	let path = cx.data_path.join(FILE_NAME);
	let raw = fs::read_to_string(&path).map_err(|e| (e, &path))?;
	let conf: Config = serde_json::from_str(&raw).map_err(|e| (e, &path))?;

	Ok(conf.parse())
}

// FIXME
pub fn prompt(cx: Context) -> Result<(), ExternalDataError> {
	let api_key = prompt_user_for("Twitter API key: ")?;
	let api_secret = prompt_user_for("Twitter API secret: ")?;
	let path = cx.data_path.join(FILE_NAME);

	fs::write(
		&path,
		serde_json::to_string(&Config::unparse(api_key, api_secret)).map_err(|e| (e, &path))?,
	)?;

	Ok(())
}
