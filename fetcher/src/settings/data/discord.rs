/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::prompt_user_for;
use crate::settings::context::StaticContext as Context;
use fetcher_config::{jobs::external_data::ExternalDataError, settings::Discord as Config};

use color_eyre::{eyre::WrapErr, Result};
use std::fs;

const FILE_NAME: &str = "discord.json";

pub fn get(cx: Context) -> Result<String, ExternalDataError> {
	let path = cx.data_path.join(FILE_NAME);
	let raw = fs::read_to_string(&path).map_err(|e| (e, &path))?;
	let conf: Config = serde_json::from_str(&raw).map_err(|e| (e, &path))?;

	Ok(conf.parse())
}

pub fn prompt(cx: Context) -> Result<()> {
	let token = prompt_user_for("Discord bot API token: ")?;
	let path = cx.data_path.join(FILE_NAME);

	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}

	fs::write(
		&path,
		serde_json::to_string(&Config::unparse(token))
			.expect("Config should always serialize to JSON without issues"),
	)
	.wrap_err_with(|| path.to_string_lossy().into_owned())?;

	Ok(())
}
