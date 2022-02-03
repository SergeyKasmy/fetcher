use serde::Deserialize;
use serde_json::Value;
use std::time::{Duration, Instant};

use crate::error::Result;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/token";

#[derive(Deserialize)]
struct GoogleOAuth2Responce {
	access_token: String,
	expires_in: u64,
}

#[derive(Debug)]
pub(crate) struct GoogleAuth {
	client_id: String,
	client_secret: String,
	refresh_token: String,
	access_token: String,
	expires_in: Instant,
}

impl GoogleAuth {
	pub(crate) async fn new(
		client_id: String,
		client_secret: String,
		refresh_token: String,
	) -> Result<Self> {
		let GoogleOAuth2Responce {
			access_token,
			expires_in,
		} = Self::generate_access_token(&client_id, &client_secret, &refresh_token).await?;

		Ok(Self {
			client_id,
			client_secret,
			refresh_token,
			access_token,
			expires_in: Instant::now() + Duration::from_secs(expires_in),
		})
	}

	pub(crate) async fn generate_refresh_token(
		client_id: &str,
		client_secret: &str,
		access_code: &str,
	) -> Result<String> {
		let body = [
			("client_id", client_id),
			("client_secret", client_secret),
			("code", access_code),
			("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
			("grant_type", "authorization_code"),
		];

		let response = reqwest::Client::new()
			.post(GOOGLE_AUTH_URL)
			.form(&body)
			.send()
			.await
			.unwrap()
			.text()
			.await
			.unwrap();

		let response: Value = serde_json::from_str(&response).unwrap();
		Ok(response
			.get("refresh_token")
			.unwrap()
			.as_str()
			.unwrap()
			.to_string())
	}

	/// Returns (refresh_token, access_token, access_token_valid_till)
	async fn generate_access_token(
		client_id: &str,
		client_secret: &str,
		refresh_token: &str,
	) -> Result<GoogleOAuth2Responce> {
		let body = [
			("client_id", client_id),
			("client_secret", client_secret),
			("refresh_token", refresh_token),
			("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
			("grant_type", "refresh_token"),
		];

		let resp: GoogleOAuth2Responce = serde_json::from_str(
			&reqwest::Client::new()
				.post(GOOGLE_AUTH_URL)
				.form(&body)
				.send()
				.await
				.unwrap()
				.text()
				.await
				.unwrap(),
		)
		.unwrap();

		Ok(resp)
	}

	pub(crate) async fn access_token(&mut self) -> Result<&str> {
		if Instant::now()
			.checked_duration_since(self.expires_in)
			.is_some()
		{
			let GoogleOAuth2Responce {
				access_token,
				expires_in,
			} = Self::generate_access_token(&self.client_id, &self.client_secret, &self.refresh_token)
				.await?;

			// self.code = CodeType::RefreshToken(refresh_token);
			self.access_token = access_token;
			self.expires_in = Instant::now() + Duration::from_secs(expires_in);
		}

		Ok(self.access_token.as_str())
	}
}
