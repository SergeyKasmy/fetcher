/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::get_data_file;
use super::prompt_user_for;
use super::write_to_data_file;
use fetcher_config::settings::Google as Config;
use fetcher_core as fcore;

use color_eyre as eyre;
use std::io;

const FILE_NAME: &str = "google_oauth2.json";

pub fn get() -> io::Result<Option<fcore::auth::Google>> {
	let raw = match get_data_file(FILE_NAME)? {
		Some(d) => d,
		None => return Ok(None),
	};

	let conf: Config = serde_json::from_str(&raw)?;

	Ok(Some(conf.parse()))
}

pub async fn prompt() -> eyre::Result<()> {
	const SCOPE: &str = "https://mail.google.com/";

	let client_id = prompt_user_for("Google OAuth2 client id: ")?;
	let client_secret = prompt_user_for("Google OAuth2 client secret: ")?;
	let access_code = prompt_user_for(&format!("Open the link below and paste the access code:\nhttps://accounts.google.com/o/oauth2/auth?scope={SCOPE}&client_id={client_id}&response_type=code&redirect_uri=urn:ietf:wg:oauth:2.0:oob\nAccess code: "))?;
	let refresh_token =
		fcore::auth::Google::generate_refresh_token(&client_id, &client_secret, &access_code)
			.await?;

	let gauth = fcore::auth::Google::new(client_id, client_secret, refresh_token);

	Ok(write_to_data_file(
		FILE_NAME,
		&serde_json::to_string(&Config::unparse(gauth))?,
	)?)
}
