/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::prompt_user_for;
use crate::settings::DATA_PATH;
use fetcher_config::settings::EmailPassword as Config;

use std::{fs, io};

const FILE_NAME: &str = "email_password.json";

pub fn get() -> io::Result<Option<String>> {
	let path = DATA_PATH.get().unwrap().join(FILE_NAME);
	let raw = fs::read_to_string(path)?;
	let conf: Config = serde_json::from_str(&raw)?;

	Ok(Some(conf.parse()))
}

pub fn prompt() -> io::Result<()> {
	let pass = prompt_user_for("Email password")?;

	fs::write(
		DATA_PATH.get().unwrap().join(FILE_NAME),
		&serde_json::to_string(&Config::unparse(pass))?,
	)?;

	Ok(())
}
