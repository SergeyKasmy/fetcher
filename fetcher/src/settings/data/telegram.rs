/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::get_data_file;
use super::prompt_user_for;
use super::write_to_data_file;
use fetcher_config::settings::Telegram as Config;

use std::io;

const FILE_NAME: &str = "telegram.json";

pub fn get() -> io::Result<Option<String>> {
	let raw = match get_data_file(FILE_NAME)? {
		Some(d) => d,
		None => return Ok(None),
	};

	let conf: Config = serde_json::from_str(&raw)?;

	Ok(Some(conf.parse()))
}

pub fn prompt() -> io::Result<()> {
	let token = prompt_user_for("Telegram bot API token: ")?;

	write_to_data_file(FILE_NAME, &serde_json::to_string(&Config::unparse(token))?)
}
