use serde::Deserialize;
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::settings;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/token";

#[derive(Debug)]
pub enum CodeType {
	AccessCode(String),
	RefreshToken(String),
}

#[derive(Deserialize)]
struct GoogleOAuth2Responce {
	access_token: String,
	expires_in: u64,
	refresh_token: Option<String>,
}

#[derive(Debug)]
pub(super) struct GoogleOAuth2 {
	pub email: String,
	client_id: String,
	client_secret: String,
	code: CodeType,
	access_token: String,
	expires_in: Instant,
}

impl GoogleOAuth2 {
	pub(super) async fn new(
		// TODO: idk how to pass this more seamlessly
		source_name: &str,
		email: String,
		client_id: String,
		client_secret: String,
		code: CodeType,
	) -> Result<Self> {
		let GoogleOAuth2Responce {
			refresh_token,
			access_token,
			expires_in,
		} = Self::generate_access_token(source_name, &client_id, &client_secret, &code).await?;

		Ok(Self {
			email,
			client_id,
			client_secret,
			code: match refresh_token {
				Some(token) => CodeType::RefreshToken(token),
				None => code,
			},
			access_token,
			expires_in: Instant::now() + Duration::from_secs(expires_in),
		})
	}

	/// Returns (refresh_token, access_token, access_token_valid_till)
	async fn generate_access_token(
		source_name: &str,
		client_id: &str,
		client_secret: &str,
		code: &CodeType,
	) -> Result<GoogleOAuth2Responce> {
		let mut body = vec![
			("client_id", client_id),
			("client_secret", client_secret),
			("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
		];

		match code {
			CodeType::AccessCode(c) => {
				body.push(("code", c));
				body.push(("grant_type", "authorization_code"));
			}
			CodeType::RefreshToken(c) => {
				body.push(("refresh_token", c));
				body.push(("grant_type", "refresh_token"));
			}
		}

		dbg!(&body);

		let resp: GoogleOAuth2Responce = serde_json::from_str(&dbg!(reqwest::Client::new()
			.post(GOOGLE_AUTH_URL)
			.form(&body)
			.send()
			.await
			.unwrap()
			.text()
			.await
			.unwrap()))
		.unwrap();

		if let Some(token) = &resp.refresh_token {
			settings::save_token(source_name, token)?;
		}
		Ok(resp)
	}

	pub(super) async fn refresh_access_token(&mut self, source_name: &str) -> Result<()> {
		if Instant::now()
			.checked_duration_since(self.expires_in)
			.is_some()
		{
			let GoogleOAuth2Responce {
				refresh_token: _,
				access_token,
				expires_in,
			} = Self::generate_access_token(
				source_name,
				&self.client_id,
				&self.client_secret,
				&self.code,
			)
			.await?;

			// self.code = CodeType::RefreshToken(refresh_token);
			self.access_token = access_token;
			self.expires_in = Instant::now() + Duration::from_secs(expires_in);
		}

		Ok(())
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
