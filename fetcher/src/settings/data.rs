/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod email_password;
pub mod google_oauth2;
pub mod telegram;
pub mod twitter;

use super::proj_dirs;

use color_eyre::Result;

use std::{
	io::{self, Write},
	path::PathBuf,
};

pub fn prompt_user_for(prompt: &str) -> io::Result<String> {
	print!("{prompt}");
	io::stdout().flush()?;

	let mut input = String::new();
	io::stdin().read_line(&mut input)?;

	Ok(input.trim().to_owned())
}

pub fn default_data_path() -> Result<PathBuf> {
	Ok(proj_dirs()?.data_dir().to_path_buf())
}
