/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::GoogleOAuth2Error;

use serde::Deserialize;
use std::time::{Duration, Instant};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/token";

#[derive(Deserialize)]
struct GoogleOAuth2Responce {
	access_token: String,
	expires_in: u64,
}

#[derive(Clone, Debug)]
struct AccessToken {
	token: String,
	expires: Instant,
}

#[allow(clippy::doc_markdown)]
/// Google OAuth2 authenticator
// TODO: link docs to the oauth2 spec
#[derive(Clone, Debug)]
pub struct Google {
	/// OAuth2 client id
	pub client_id: String,
	/// OAuth2 client secret
	pub client_secret: String,
	/// OAuth2 refresh token
	pub refresh_token: String,
	access_token: Option<AccessToken>,
}

impl Google {
	#[allow(clippy::doc_markdown)]
	/// Creates a new Google OAuth2 authenticator
	#[must_use]
	pub fn new(client_id: String, client_secret: String, refresh_token: String) -> Self {
		Self {
			client_id,
			client_secret,
			refresh_token,
			access_token: None,
		}
	}

	#[allow(clippy::doc_markdown)]
	/// Generate a new Google OAuth2 refresh token using the `client_id`, `client_secret`, and `access_code`
	///
	/// # Errors
	/// * if there was a network connection error
	/// * if the responce isn't a valid refresh_token
	pub async fn generate_refresh_token(
		client_id: &str,
		client_secret: &str,
		access_code: &str,
	) -> Result<String, GoogleOAuth2Error> {
		#[derive(Deserialize)]
		struct Response {
			refresh_token: String,
		}

		let body = [
			("client_id", client_id),
			("client_secret", client_secret),
			("code", access_code),
			("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
			("grant_type", "authorization_code"),
		];

		let resp = reqwest::Client::new()
			.post(GOOGLE_AUTH_URL)
			.form(&body)
			.send()
			.await
			.map_err(GoogleOAuth2Error::Post)?
			.text()
			.await
			.map_err(GoogleOAuth2Error::Post)?;

		let Response { refresh_token } =
			serde_json::from_str(&resp).map_err(|_| GoogleOAuth2Error::Auth(resp))?;
		Ok(refresh_token)
	}

	async fn generate_access_token(
		client_id: &str,
		client_secret: &str,
		refresh_token: &str,
	) -> Result<GoogleOAuth2Responce, GoogleOAuth2Error> {
		let body = [
			("client_id", client_id),
			("client_secret", client_secret),
			("refresh_token", refresh_token),
			("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
			("grant_type", "refresh_token"),
		];

		let resp = reqwest::Client::new()
			.post(GOOGLE_AUTH_URL)
			.form(&body)
			.send()
			.await
			.map_err(GoogleOAuth2Error::Post)?
			.text()
			.await
			.map_err(GoogleOAuth2Error::Post)?;

		// TODO: maybe use the result from serde instead of the responce itself?
		serde_json::from_str(&resp).map_err(|_| GoogleOAuth2Error::Auth(resp))
	}

	async fn validate_access_token(&mut self) -> Result<(), GoogleOAuth2Error> {
		let GoogleOAuth2Responce {
			access_token,
			expires_in,
		} = Self::generate_access_token(&self.client_id, &self.client_secret, &self.refresh_token)
			.await?;

		self.access_token = Some(AccessToken {
			token: access_token,
			expires: Instant::now() + Duration::from_secs(expires_in - /* buffer */ 5), // add 5 seconds as buffer since some time could've passed since the server issued the token
		});

		Ok(())
	}

	/// Return a previously gotten `access_token` or fetch a new one
	///
	/// # Errors
	/// * if there was a network connection error
	/// * if the responce isn't a valid `refresh_token`
	#[allow(clippy::missing_panics_doc)] // this should never panic
	pub async fn access_token(&mut self) -> Result<&str, GoogleOAuth2Error> {
		// FIXME: for some reason the token sometimes expires by itself and should be renewed manually
		// Update the token if:
		// we haven't done that yet
		if self.access_token.is_none()
			// or if if has expired
			|| self
				.access_token
				.as_ref()
				.and_then(|x| Instant::now().checked_duration_since(x.expires))
				.is_some()
		{
			self.validate_access_token().await?;
		}

		Ok(self
			.access_token
			.as_ref()
			.map(|x| x.token.as_str())
			.expect("Token should have just been validated and thus be present and valid"))
	}
}
