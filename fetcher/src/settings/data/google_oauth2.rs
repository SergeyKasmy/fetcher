/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::prompt_user_for;
use crate::settings::context::StaticContext as Context;
use fetcher_config::{jobs::external_data::ExternalDataError, settings::Google as Config};
use fetcher_core as fcore;

use color_eyre as eyre;
use std::fs;

const FILE_NAME: &str = "google_oauth2.json";

pub fn get(cx: Context) -> Result<fcore::auth::Google, ExternalDataError> {
	let path = cx.data_path.join(FILE_NAME);
	let raw = fs::read_to_string(&path).map_err(|e| (e, &path))?;
	let conf: Config = serde_json::from_str(&raw).map_err(|e| (e, &path))?;

	Ok(conf.parse())
}

pub async fn prompt(cx: Context) -> eyre::Result<()> {
	const SCOPE: &str = "https://mail.google.com/";

	let client_id = prompt_user_for("Google OAuth2 client id: ")?;
	let client_secret = prompt_user_for("Google OAuth2 client secret: ")?;
	let access_code = prompt_user_for(&format!("Open the link below and paste the access code:\nhttps://accounts.google.com/o/oauth2/auth?scope={SCOPE}&client_id={client_id}&response_type=code&redirect_uri=urn:ietf:wg:oauth:2.0:oob\nAccess code: "))?;
	let refresh_token =
		fcore::auth::google::generate_refresh_token(&client_id, &client_secret, &access_code)
			.await?;

	let gauth = fcore::auth::Google::new(client_id, client_secret, refresh_token);

	fs::write(
		cx.data_path.join(FILE_NAME),
		serde_json::to_string(&Config::unparse(gauth))?,
	)?;

	Ok(())
}
