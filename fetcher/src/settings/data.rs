/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::PREFIX;
use fetcher_core::auth as core_auth;

use color_eyre::Result;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{fs, io};

const GOOGLE_OAUTH2: &str = "google_oauth2.json";
const EMAIL_PASS: &str = "email_pass.txt";
const TWITTER: &str = "twitter.json";
const TELEGRAM: &str = "telegram.txt";

fn data_path(name: &str) -> Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/{name}"))
	} else {
		xdg::BaseDirectories::with_prefix(PREFIX)?.place_data_file(name)?
	})
}

async fn input(prompt: &str, expected_input_len: usize) -> Result<String> {
	use std::io::Write;

	print!("{prompt}");
	std::io::stdout().flush()?;

	let mut buf = String::with_capacity(expected_input_len);
	BufReader::new(io::stdin()).read_line(&mut buf).await?;

	Ok(buf.trim().to_string())
}

pub async fn data(name: &str) -> Result<Option<String>> {
	let f = data_path(name)?;
	if !f.is_file() {
		return Ok(None);
	}

	Ok(Some(fs::read_to_string(&f).await?))
}

#[allow(clippy::doc_markdown)]
/// Get date required for authentication with Google OAuth2
///
/// # Errors
/// * if the file is inaccessible
/// * if the file is corrupted
pub async fn google_oauth2() -> Result<Option<core_auth::Google>> {
	let data = match data(GOOGLE_OAUTH2).await? {
		Some(d) => d,
		None => return Ok(None),
	};

	let conf: fetcher_config::settings::Google = serde_json::from_str(&data)?;

	Ok(Some(conf.parse()))
}

pub async fn email_password() -> Result<Option<String>> {
	data(EMAIL_PASS).await
}

pub async fn twitter() -> Result<Option<(String, String)>> {
	let data = match data(TWITTER).await? {
		Some(d) => d,
		None => return Ok(None),
	};

	let twitter: fetcher_config::settings::Twitter = serde_json::from_str(&data)?;

	Ok(Some(twitter.parse()))
}

pub async fn telegram() -> Result<Option<String>> {
	data(TELEGRAM).await
}

async fn save_data(name: &str, data: &str) -> Result<()> {
	fs::write(&data_path(name)?, data).await?;

	Ok(())
}

pub async fn prompt_google_oauth2() -> Result<()> {
	const SCOPE: &str = "https://mail.google.com/";

	let client_id = input("Google OAuth2 client id: ", 100).await?;
	let client_secret = input("Google OAuth2 client secret: ", 40).await?;
	let access_code = input(&format!("Open the link below and paste the access code:\nhttps://accounts.google.com/o/oauth2/auth?scope={SCOPE}&client_id={client_id}&response_type=code&redirect_uri=urn:ietf:wg:oauth:2.0:oob\nAccess code: "), 75).await?;
	let refresh_token =
		core_auth::Google::generate_refresh_token(&client_id, &client_secret, &access_code).await?;

	let gauth = core_auth::Google::new(client_id, client_secret, refresh_token);

	save_data(
		GOOGLE_OAUTH2,
		&serde_json::to_string(&fetcher_config::settings::Google::unparse(gauth))?,
	)
	.await
}

pub async fn prompt_email_password() -> Result<()> {
	let pass = input("Email password", 25).await?;

	save_data(
		EMAIL_PASS,
		&serde_json::to_string(&fetcher_config::settings::EmailPassword::unparse(pass))?,
	)
	.await
}

pub async fn prompt_twitter_auth() -> Result<()> {
	let api_key = input("Twitter API key: ", 25).await?;
	let api_secret = input("Twitter API secret: ", 50).await?;

	save_data(
		TWITTER,
		&serde_json::to_string(&fetcher_config::settings::Twitter::unparse(
			api_key, api_secret,
		))?,
	)
	.await
}

pub async fn prompt_telegram() -> Result<()> {
	let token = input("Telegram bot API token: ", 50).await?;
	save_data(
		"telegram.txt",
		&serde_json::to_string(&fetcher_config::settings::Telegram::unparse(token))?,
	)
	.await
}
