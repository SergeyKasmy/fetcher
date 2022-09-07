/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod email_password;
pub mod google_oauth2;
pub mod telegram;
pub mod twitter;

use super::PREFIX;

use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;

const EMAIL_PASS: &str = "email_pass.txt";

pub fn get_data_file(file_name: &str) -> io::Result<Option<String>> {
	let f = get_data_file_path(file_name)?;
	if !f.is_file() {
		return Ok(None);
	}

	Ok(Some(fs::read_to_string(&f)?))
}

pub fn prompt_user_for(prompt: &str) -> io::Result<String> {
	print!("{prompt}");
	io::stdout().flush()?;

	let mut input = String::new();
	io::stdin().read_line(&mut input)?;

	Ok(input.trim().to_owned())
}

pub fn write_to_data_file(file_name: &str, data: &str) -> io::Result<()> {
	fs::write(&get_data_file_path(file_name)?, data)
}

fn get_data_file_path(name: &str) -> io::Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/{name}"))
	} else {
		xdg::BaseDirectories::with_prefix(PREFIX)?.place_data_file(name)?
	})
}
