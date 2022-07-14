/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;
use std::time::{Duration, Instant};

use crate::error::GoogleOAuth2Error;

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

#[derive(Clone, Debug)]
pub struct Google {
	client_id: String,
	client_secret: String,
	refresh_token: String,
	access_token: Option<AccessToken>,
}

impl Google {
	#[must_use]
	pub fn new(client_id: String, client_secret: String, refresh_token: String) -> Self {
		Self {
			client_id,
			client_secret,
			refresh_token,
			access_token: None,
		}
	}

	#[allow(clippy::items_after_statements)] // TODO
	pub async fn generate_refresh_token(
		client_id: &str,
		client_secret: &str,
		access_code: &str,
	) -> Result<String, GoogleOAuth2Error> {
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

		// TODO: find a better way to get a string without a temporary struct or a million of ok_or()'s
		#[derive(Deserialize)]
		struct Response {
			refresh_token: String,
		}

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
			expires: Instant::now() + Duration::from_secs(expires_in),
		});

		Ok(())
	}

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

		// unwrap NOTE: should be safe, we just validated it up above
		Ok(self
			.access_token
			.as_ref()
			.map(|x| x.token.as_str())
			.unwrap())
	}
}
