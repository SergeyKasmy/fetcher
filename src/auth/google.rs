/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// I can avoid the clippy::doc_markdown lint this way :P
#![doc = "This module contains the Google authenticator that can access Google services via OAuth2"]

use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};

use crate::{StaticStr, error::Error};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/token";
const TOKEN_REFRESH_BUFFER: Duration = Duration::from_secs(60);

#[expect(clippy::doc_markdown, reason = "false positive")]
/// Google OAuth2 authenticator
#[derive(Debug)]
pub struct Google {
	/// OAuth2 client id
	pub client_id: StaticStr,

	/// OAuth2 client secret
	pub client_secret: StaticStr,

	/// OAuth2 refresh token. It doesn't expire and is used to get new shortlived access tokens
	pub refresh_token: StaticStr,

	/// OAuth2 access token. It's used for the actual accessing of the data
	access_token: Mutex<Option<AccessToken>>,
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum GoogleOAuth2Error {
	#[error("Error contacting Google servers for authentication")]
	Post(#[source] reqwest::Error),

	#[error("Can't get a new OAuth2 refresh token from Google: {0}")]
	RefreshToken(String),

	#[error("Can't get a new OAuth2 access token from Google: {0}")]
	AccessToken(String),
}

#[expect(clippy::doc_markdown, reason = "false positive")]
/// An OAuth2 access token. It can be used to actually access stuff via OAuth2
#[derive(Clone, Debug)]
struct AccessToken {
	/// The token itself
	pub token: String,

	/// When it expires and will no longer be valid
	pub expires: Instant,
}

#[derive(Deserialize)]
struct AccessTokenResponce {
	access_token: String,
	expires_in: u64,
}

impl Google {
	#[expect(clippy::doc_markdown, reason = "false positive")]
	/// Creates a new Google OAuth2 authenticator
	#[must_use]
	pub fn new(
		client_id: impl Into<StaticStr>,
		client_secret: impl Into<StaticStr>,
		refresh_token: impl Into<StaticStr>,
	) -> Self {
		Self {
			client_id: client_id.into(),
			client_secret: client_secret.into(),
			refresh_token: refresh_token.into(),
			access_token: Mutex::new(None),
		}
	}

	/// Return a previously gotten `access_token` or fetch a new one
	///
	/// # Errors
	/// * if there was a network connection error
	/// * if the responce isn't a valid `refresh_token`
	#[tracing::instrument(name = "google_oauth2_access_token")]
	pub async fn access_token(&mut self) -> Result<MappedMutexGuard<'_, str>, GoogleOAuth2Error> {
		let mut access_token = self.access_token.lock().await;

		// Update the token if:
		if {
			// we haven't done that yet
			let access_token_doesnt_exist = access_token.is_none();
			if access_token_doesnt_exist {
				tracing::trace!("Access token doesn't exist");
			}

			access_token_doesnt_exist
		} || {
			// or if it expires within the buffer time
			let should_refresh = access_token
				.as_ref()
				.map(|x| x.expires.duration_since(Instant::now()) <= TOKEN_REFRESH_BUFFER)
				.unwrap_or(true);

			if should_refresh {
				tracing::trace!("Access token needs refresh (expires soon or doesn't exist)");
			}

			should_refresh
		} {
			let new_access_token =
				generate_access_token(&self.client_id, &self.client_secret, &self.refresh_token)
					.await?;

			let expires = Instant::now() + Duration::from_secs(new_access_token.expires_in);

			tracing::debug!(
				"Access token is still valid for {:?}s",
				expires
					.checked_duration_since(Instant::now())
					.map(|dur| dur.as_secs())
			);

			*access_token = Some(AccessToken {
				token: new_access_token.access_token,
				expires,
			});
		}

		Ok(MutexGuard::map(access_token, |token| {
			&mut *token
				.as_mut()
				.expect("Token should've just been validated to be present and valid")
				.token
		}))
	}
}

impl Error for GoogleOAuth2Error {
	fn is_network_related(&self) -> Option<&dyn Error> {
		match self {
			GoogleOAuth2Error::Post(_) => Some(self),
			_ => None,
		}
	}
}

#[expect(clippy::doc_markdown, reason = "false positive")]
/// Generate and return a new Google OAuth2 refresh token using the `client_id`, `client_secret`, and `access_code`
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

	tracing::debug!(
		"Generating a new OAuth2 refresh token from client_id: {client_id:?}, client_secret: {client_secret:?}, and access_code: {access_code:?}"
	);

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

	tracing::debug!("Got {resp:?} from the Google OAuth2 endpoint");

	let Response { refresh_token } =
		serde_json::from_str(&resp).map_err(|_| GoogleOAuth2Error::RefreshToken(resp))?;

	Ok(refresh_token)
}

async fn generate_access_token(
	client_id: &str,
	client_secret: &str,
	refresh_token: &str,
) -> Result<AccessTokenResponce, GoogleOAuth2Error> {
	tracing::debug!(
		"Generating a new OAuth2 access token from client_id: {client_id:?}, client_secret: {client_secret:?}, and refresh_token: {refresh_token:?}"
	);

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

	tracing::debug!("Got {resp:?} from the Google OAuth2 endpoint");

	serde_json::from_str(&resp).map_err(|_| GoogleOAuth2Error::AccessToken(resp))
}
