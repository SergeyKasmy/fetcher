/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use teloxide::Bot;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{fs, io};

use super::PREFIX;
use crate::config;
use crate::error::ConfigError;
use fetcher_core::auth;

const GOOGLE_OAUTH2: &str = "google_oauth2.json";
const GOOGLE_PASS: &str = "google_pass.txt";
const TWITTER: &str = "twitter.json";
const TELEGRAM: &str = "telegram.txt";

#[derive(Serialize, Deserialize)]
struct TwitterAuthSaveFormat {
	api_key: String,
	api_secret: String,
}

fn data_path(name: &str) -> Result<PathBuf, ConfigError> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/{name}"))
	} else {
		xdg::BaseDirectories::with_prefix(PREFIX)?
			.place_data_file(name)
			.map_err(|e| ConfigError::Read(e, name.into()))?
	})
}

async fn input(prompt: &str, expected_input_len: usize) -> Result<String, ConfigError> {
	use std::io::Write;

	print!("{prompt}");
	std::io::stdout()
		.flush()
		.map_err(ConfigError::StdoutWrite)?; // TODO: why can't I use tokio version?

	let mut buf = String::with_capacity(expected_input_len);
	BufReader::new(io::stdin())
		.read_line(&mut buf)
		.await
		.map_err(ConfigError::StdinRead)?;

	Ok(buf.trim().to_string())
}

pub(crate) async fn data(name: &str) -> Result<Option<String>, ConfigError> {
	let f = data_path(name)?;
	if !f.is_file() {
		return Ok(None);
	}

	Some(
		fs::read_to_string(&f)
			.await
			.map_err(|e| ConfigError::Read(e, f)),
	)
	.transpose()
}

#[allow(clippy::doc_markdown)] // TODO
/// Get date required for authentication with Google OAuth2
///
/// # Errors
/// * if the file is inaccessible
/// * if the file is corrupted
pub(crate) async fn google_oauth2() -> Result<Option<auth::Google>, ConfigError> {
	let data = match data(GOOGLE_OAUTH2).await? {
		Some(d) => d,
		None => return Ok(None),
	};

	let conf: config::auth::Google = serde_json::from_str(&data)
		.map_err(|e| ConfigError::CorruptedConfig(Box::new(e), GOOGLE_OAUTH2.into()))?;

	Ok(Some(conf.parse()))
}

/// TODO: rename to email password
pub(crate) async fn google_password() -> Result<Option<String>, ConfigError> {
	data(GOOGLE_PASS).await
}

pub(crate) async fn twitter() -> Result<Option<(String, String)>, ConfigError> {
	let data = match data(TWITTER).await? {
		Some(d) => d,
		None => return Ok(None),
	};

	let TwitterAuthSaveFormat {
		api_key,
		api_secret,
	} = serde_json::from_str(&data)
		.map_err(|e| ConfigError::CorruptedConfig(Box::new(e), TWITTER.into()))?;

	Ok(Some((api_key, api_secret)))
}

pub(crate) async fn telegram() -> Result<Option<Bot>, ConfigError> {
	Ok(data(TELEGRAM).await?.map(Bot::new))
}

async fn save_data(name: &str, data: &str) -> Result<(), ConfigError> {
	let p = data_path(name)?;
	fs::write(&p, data)
		.await
		.map_err(|e| ConfigError::Write(e, p))
}

pub(crate) async fn generate_google_oauth2() -> Result<(), ConfigError> {
	const SCOPE: &str = "https://mail.google.com/";

	let client_id = input("Google OAuth2 client id: ", 100).await?;
	let client_secret = input("Google OAuth2 client secret: ", 40).await?;
	let access_code = input(&format!("Open the link below and paste the access code:\nhttps://accounts.google.com/o/oauth2/auth?scope={SCOPE}&client_id={client_id}&response_type=code&redirect_uri=urn:ietf:wg:oauth:2.0:oob\nAccess code: "), 75).await?;
	let refresh_token =
		auth::Google::generate_refresh_token(&client_id, &client_secret, &access_code).await?;

	save_data(
		GOOGLE_OAUTH2,
		&serde_json::to_string(&config::auth::Google {
			client_id,
			client_secret,
			refresh_token,
		})
		.unwrap(), // NOTE: shouldn't fail, these are just strings
	)
	.await
}

// TODO: maybe "generate" isn't the best word?
pub(crate) async fn generate_google_password() -> Result<(), ConfigError> {
	let pass = input("Google app password", 25).await?;

	save_data(GOOGLE_PASS, &pass).await
}

pub(crate) async fn generate_twitter_auth() -> Result<(), ConfigError> {
	let api_key = input("Twitter API key: ", 25).await?;
	let api_secret = input("Twitter API secret: ", 50).await?;

	save_data(
		TWITTER,
		&serde_json::to_string(&TwitterAuthSaveFormat {
			api_key,
			api_secret,
		})
		.unwrap(), // NOTE: shouldn't fail, these are just strings
	)
	.await
}

pub(crate) async fn generate_telegram() -> Result<(), ConfigError> {
	let key = input("Telegram bot API key: ", 50).await?;
	save_data("telegram.txt", &key).await
}
