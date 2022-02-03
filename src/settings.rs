use std::{fs, path::PathBuf};

use crate::{
	auth::GoogleAuth,
	error::{Error, Result},
};

const PREFIX: &str = "fetcher";
const CONFIG: &str = "config.toml";
const LAST_READ_DATA_DIR: &str = "last-read";

pub fn get_config() -> Result<String> {
	let path = if !cfg!(debug_assertions) {
		xdg::BaseDirectories::with_prefix(PREFIX)
			.map_err(|e| Error::GetConfig(e.to_string()))?
			.place_config_file(CONFIG)
			.map_err(|e| Error::GetConfig(e.to_string()))?
	} else {
		PathBuf::from(format!("debug_data/{CONFIG}"))
	};

	fs::read_to_string(&path).map_err(|e| Error::GetConfig(e.to_string()))
}

fn last_read_id_path(name: &str) -> Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/last-read-id-{name}"))
	} else {
		xdg::BaseDirectories::with_profile(PREFIX, LAST_READ_DATA_DIR)
			.map_err(|e| Error::GetData(e.to_string()))?
			.place_data_file(name)
			.map_err(|e| Error::GetData(e.to_string()))?
	})
}

pub fn last_read_id(name: &str) -> Result<Option<String>> {
	Ok(fs::read_to_string(last_read_id_path(name)?).ok())
}

pub fn save_last_read_id(name: &str, id: String) -> Result<()> {
	fs::write(last_read_id_path(name)?, id).map_err(|e| Error::SaveData(e.to_string()))
}

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

pub fn data(name: &str) -> Result<Option<String>> {
	Ok(fs::read_to_string(data_path(name)?).ok())
}

pub fn save_data(name: &str, data: &str) -> Result<()> {
	fs::write(data_path(name)?, data).map_err(|e| Error::SaveData(e.to_string()))
}

pub async fn generate_google_oauth2() -> Result<()> {
	use std::io::stdin;
	const SCOPE: &str = "https://mail.google.com/";

	// FIXME: update the capacity after testing
	let mut client_id = String::with_capacity(50);
	println!("Google OAuth2 client id: ");
	stdin().read_line(&mut client_id).unwrap();
	let client_id = client_id.trim().to_string();

	let mut client_secret = String::with_capacity(50);
	println!("Google OAuth2 client secret: ");
	stdin().read_line(&mut client_secret).unwrap();
	let client_secret = client_secret.trim().to_string();

	let mut access_code = String::with_capacity(50);
	println!("Open the link below and paste the access code afterwards:");
	println!("https://accounts.google.com/o/oauth2/auth?scope={SCOPE}&client_id={client_id}&response_type=code&redirect_uri=urn:ietf:wg:oauth:2.0:oob");
	stdin().read_line(&mut access_code).unwrap();
	let access_code = access_code.trim().to_string();
	// let token = crate::source::email::google_oauth2::generate_refresh_token(
	let refresh_token =
		GoogleAuth::generate_refresh_token(&client_id, &client_secret, &access_code).await?;

	let auth = GoogleAuth::new(client_id, client_secret, refresh_token).await?;
	save_data("google_oauth2.json", &serde_json::to_string(&auth).unwrap())
}
