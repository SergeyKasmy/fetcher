/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::{
	fs,
	io::{self, stdin, Write},
	path::PathBuf,
};

use serde::{Deserialize, Serialize};
use teloxide::Bot;

use super::PREFIX;
use crate::{
	auth::GoogleAuth,
	error::{Error, Result},
};

const GOOGLE_OAUTH2: &str = "google_oauth2.json";
const GOOGLE_PASS: &str = "google_pass.txt";
const TWITTER: &str = "twitter.json";
const TELEGRAM: &str = "telegram.txt";

#[derive(Serialize, Deserialize)]
struct TwitterAuthSaveFormat {
	api_key: String,
	api_secret: String,
}

// TODO: dedup with last_read_id_path
fn data_path(name: &str) -> Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/{name}"))
	} else {
		xdg::BaseDirectories::with_prefix(PREFIX)?
			.place_data_file(name)
			.map_err(|e| Error::InaccessibleData(e, name.into()))?
	})
}

fn input(prompt: &str, expected_input_len: usize) -> Result<String> {
	print!("{prompt}");
	io::stdout().flush().map_err(Error::Stdout)?;

	let mut buf = String::with_capacity(expected_input_len);
	stdin().read_line(&mut buf).map_err(Error::Stdin)?;

	Ok(buf.trim().to_string())
}

fn data(name: &str) -> Result<String> {
	let f = data_path(name)?;
	fs::read_to_string(&f).map_err(|e| Error::InaccessibleData(e, f))
}

pub fn google_oauth2() -> Result<GoogleAuth> {
	serde_json::from_str(&data(GOOGLE_OAUTH2)?)
		.map_err(|e| Error::CorruptedData(e, GOOGLE_OAUTH2.into()))
}

pub fn google_password() -> Result<String> {
	data(GOOGLE_PASS)
}

pub fn twitter() -> Result<(String, String)> {
	let TwitterAuthSaveFormat {
		api_key,
		api_secret,
	} = serde_json::from_str(&data(TWITTER)?).map_err(|e| Error::CorruptedData(e, TWITTER.into()))?;

	Ok((api_key, api_secret))
}

pub fn telegram() -> Result<Bot> {
	Ok(Bot::new(data(TELEGRAM)?))
}

fn save_data(name: &str, data: &str) -> Result<()> {
	let p = data_path(name)?;
	fs::write(&p, data).map_err(|e| Error::Write(e, p))
}

pub async fn generate_google_oauth2() -> Result<()> {
	const SCOPE: &str = "https://mail.google.com/";

	let client_id = input("Google OAuth2 client id: ", 100)?;
	let client_secret = input("Google OAuth2 client secret: ", 40)?;
	let access_code = input(&format!("Open the link below and paste the access code:\nhttps://accounts.google.com/o/oauth2/auth?scope={SCOPE}&client_id={client_id}&response_type=code&redirect_uri=urn:ietf:wg:oauth:2.0:oob\nAccess code: "), 75)?;
	let refresh_token =
		GoogleAuth::generate_refresh_token(&client_id, &client_secret, &access_code).await?;

	save_data(
		GOOGLE_OAUTH2,
		&serde_json::to_string(&GoogleAuth::new(client_id, client_secret, refresh_token).await?)
			.unwrap(), // NOTE: shouldn't fail, these are just strings
	)
}

// TODO: maybe "generate" isn't the best word?
pub fn generate_google_password() -> Result<()> {
	let pass = input("Google app password", 25)?;

	save_data(GOOGLE_PASS, &pass)
}

pub fn generate_twitter_auth() -> Result<()> {
	let api_key = input("Twitter API key: ", 25)?;
	let api_secret = input("Twitter API secret: ", 50)?;

	save_data(
		TWITTER,
		&serde_json::to_string(&TwitterAuthSaveFormat {
			api_key,
			api_secret,
		})
		.unwrap(), // NOTE: shouldn't fail, these are just strings
	)
}

pub fn generate_telegram() -> Result<()> {
	let key = input("Telegram bot API key: ", 50)?;
	save_data("telegram.txt", &key)
}
