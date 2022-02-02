use serde::Deserialize;
use std::time::{Duration, Instant};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/token";

#[derive(Deserialize)]
struct GoogleOAuth2Responce {
	refresh_token: String,
	access_token: String,
	expires_in: u64,
}

pub(super) struct GoogleOAuth2 {
	pub email: String,
	client_id: String,
	client_secret: String,
	refresh_token: String,
	access_token: String,
	expires_in: Instant,
}

impl GoogleOAuth2 {
	pub(super) async fn new(
		email: String,
		client_id: String,
		client_secret: String,
		refresh_token: String,
	) -> Self {
		let GoogleOAuth2Responce {
			refresh_token,
			access_token,
			expires_in,
		} = Self::generate_access_token(&client_id, &client_secret, &refresh_token).await;

		Self {
			email,
			client_id,
			client_secret,
			refresh_token,
			access_token,
			expires_in: Instant::now() + Duration::from_secs(expires_in),
		}
	}

	/// Returns (refresh_token, access_token, access_token_valid_till)
	async fn generate_access_token(
		client_id: &str,
		client_secret: &str,
		refresh_token: &str,
	) -> GoogleOAuth2Responce {
		let body = [
			("client_id", client_id),
			("client_secret", client_secret),
			("code", refresh_token),
			("grant_type", "authorization_code"),
			("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
		];

		serde_json::from_str(
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
		.unwrap()
	}

	pub(super) async fn refresh_access_token(&mut self) {
		if Instant::now()
			.checked_duration_since(self.expires_in)
			.is_some()
		{
			let GoogleOAuth2Responce {
				refresh_token,
				access_token,
				expires_in,
			} = Self::generate_access_token(&self.client_id, &self.client_secret, &self.refresh_token)
				.await;

			self.refresh_token = refresh_token;
			self.access_token = access_token;
			self.expires_in = Instant::now() + Duration::from_secs(expires_in);
		}
	}
}

impl imap::Authenticator for GoogleOAuth2 {
	type Response = String;

	fn process(&self, _challenge: &[u8]) -> Self::Response {
		format!(
			"user={}\x01auth=Bearer {}\x01\x01",
			self.email, self.access_token
		)
	}
}
