use std::{
	fs,
	io::{self, stdin, Write},
	path::PathBuf,
};

use super::PREFIX;
use crate::{
	auth::GoogleAuth,
	config::formats::{GoogleAuthCfg, TwitterCfg},
	error::{Error, Result},
};

const GOOGLE_OAUTH2: &str = "google_oauth2.json";
const GOOGLE_PASS: &str = "google_pass.txt";
const TWITTER: &str = "twitter.json";
const TELEGRAM: &str = "telegram.txt";

// TODO: dedup with last_read_id_path
fn data_path(name: &str) -> Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/{name}"))
	} else {
		xdg::BaseDirectories::with_prefix(PREFIX)
			.map_err(|e| Error::GetData(e.to_string()))?
			.place_data_file(name)
			.map_err(|e| Error::GetData(e.to_string()))?
	})
}

fn input(prompt: &str, expected_input_len: usize) -> Result<String> {
	print!("{prompt}");
	io::stdout().flush()?;

	let mut buf = String::with_capacity(expected_input_len);
	stdin().read_line(&mut buf)?;

	Ok(buf.trim().to_string())
}

fn data(name: &str) -> Result<Option<String>> {
	Ok(fs::read_to_string(data_path(name)?).ok())
}

pub fn google_oauth2() -> Result<Option<GoogleAuthCfg>> {
	Ok(serde_json::from_str(&data(GOOGLE_OAUTH2)?.ok_or_else(
		|| Error::GetData("Google OAuth2 data not found".to_string()),
	)?)?)
}

pub fn google_password() -> Result<Option<String>> {
	data(GOOGLE_PASS)
}

pub fn twitter() -> Result<Option<TwitterCfg>> {
	Ok(serde_json::from_str(&data(TWITTER)?.ok_or_else(|| {
		Error::GetData("Twitter data not found".to_string())
	})?)?)
}

pub fn telegram() -> Result<Option<String>> {
	data(TELEGRAM)
}

fn save_data(name: &str, data: &str) -> Result<()> {
	fs::write(data_path(name)?, data).map_err(|e| Error::SaveData(e.to_string()))
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
		&serde_json::to_string(&GoogleAuthCfg {
			client_id,
			client_secret,
			refresh_token,
		})?,
	)
}

// TODO: maybe "generate" isn't the best word?
pub fn generate_google_password() -> Result<()> {
	let pass = input("Google app password", 25)?;

	save_data(GOOGLE_PASS, &pass)
}

pub fn generate_twitter_auth() -> Result<()> {
	let key = input("Twitter API key: ", 25)?;
	let secret = input("Twitter API secret: ", 50)?;

	save_data(
		TWITTER,
		&serde_json::to_string(&TwitterCfg { key, secret })?,
	)
}

pub fn generate_telegram() -> Result<()> {
	let key = input("Telegram bot API key: ", 50)?;
	save_data("telegram.txt", &key)
}
