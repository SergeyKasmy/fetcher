/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use fetcher::{
	auth, config,
	error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use teloxide::Bot;
// use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{fs, io};

use super::PREFIX;

const GOOGLE_OAUTH2: &str = "google_oauth2.json";
const GOOGLE_PASS: &str = "google_pass.txt";
const TWITTER: &str = "twitter.json";
const TELEGRAM: &str = "telegram.txt";

#[derive(Serialize, Deserialize)]
struct TwitterAuthSaveFormat {
	api_key: String,
	api_secret: String,
}

fn data_path(name: &str) -> Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/{name}"))
	} else {
		xdg::BaseDirectories::with_prefix(PREFIX)?
			.place_data_file(name)
			.map_err(|e| Error::LocalIoRead(e, name.into()))?
	})
}

// fn input(prompt: &str, expected_input_len: usize) -> Result<String> {
// 	use std::io::{stdin, stdout, Write};

// 	print!("{prompt}");
// 	stdout().flush().map_err(Error::Stdout)?;

// 	let mut buf = String::with_capacity(expected_input_len);
// 	stdin().read_line(&mut buf).map_err(Error::Stdin)?;

// 	Ok(buf.trim().to_string())
// }

async fn input(prompt: &str, expected_input_len: usize) -> Result<String> {
	use std::io::Write;

	print!("{prompt}");
	std::io::stdout().flush().map_err(Error::Stdout)?; // TODO: why can't I use tokio version?

	let mut buf = String::with_capacity(expected_input_len);
	BufReader::new(io::stdin())
		.read_line(&mut buf)
		.await
		.map_err(Error::Stdin)?;

	Ok(buf.trim().to_string())
}

pub async fn data(name: &str) -> Result<Option<String>> {
	let f = data_path(name)?;
	if !f.is_file() {
		return Ok(None);
	}

	Some(
		fs::read_to_string(&f)
			.await
			.map_err(|e| Error::LocalIoRead(e, f)),
	)
	.transpose()
}

#[allow(clippy::doc_markdown)] // TODO
/// Get date required for authentication with Google OAuth2
///
/// # Errors
/// * if the file is inaccessible
/// * if the file is corrupted
pub async fn google_oauth2() -> Result<Option<auth::Google>> {
	let data = match data(GOOGLE_OAUTH2).await? {
		Some(d) => d,
		None => return Ok(None),
	};

	let conf: config::auth::Google =
		serde_json::from_str(&data).map_err(|e| Error::CorruptedFile(e, GOOGLE_OAUTH2.into()))?;

	Ok(Some(conf.parse()))
}

/// TODO: rename to email password
pub async fn google_password() -> Result<Option<String>> {
	data(GOOGLE_PASS).await
}

pub async fn twitter() -> Result<Option<(String, String)>> {
	let data = match data(TWITTER).await? {
		Some(d) => d,
		None => return Ok(None),
	};

	let TwitterAuthSaveFormat {
		api_key,
		api_secret,
	} = serde_json::from_str(&data).map_err(|e| Error::CorruptedFile(e, TWITTER.into()))?;

	Ok(Some((api_key, api_secret)))
}

pub async fn telegram() -> Result<Option<Bot>> {
	Ok(data(TELEGRAM).await?.map(|s| Bot::new(s)))
}

async fn save_data(name: &str, data: &str) -> Result<()> {
	let p = data_path(name)?;
	fs::write(&p, data)
		.await
		.map_err(|e| Error::LocalIoWrite(e, p))
}

pub async fn generate_google_oauth2() -> Result<()> {
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
pub async fn generate_google_password() -> Result<()> {
	let pass = input("Google app password", 25).await?;

	save_data(GOOGLE_PASS, &pass).await
}

pub async fn generate_twitter_auth() -> Result<()> {
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

pub async fn generate_telegram() -> Result<()> {
	let key = input("Telegram bot API key: ", 50).await?;
	save_data("telegram.txt", &key).await
}
