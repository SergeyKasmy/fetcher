use crate::auth::GoogleAuth;
use crate::error::Result;

pub enum Auth {
	// TODO: use securestr or something of that sort
	Password(String),
	GoogleAuth(GoogleAuth),
}

pub(super) struct ImapOAuth2<'a> {
	email: &'a str,
	token: &'a str,
}

impl imap::Authenticator for ImapOAuth2<'_> {
	type Response = String;

	fn process(&self, _challenge: &[u8]) -> Self::Response {
		format!("user={}\x01auth=Bearer {}\x01\x01", self.email, self.token)
	}
}

#[async_trait::async_trait]
pub(super) trait GoogleAuthExt {
	async fn to_imap_oauth2<'a>(&'a mut self, email: &'a str) -> Result<ImapOAuth2<'a>>;
}

#[async_trait::async_trait]
impl GoogleAuthExt for GoogleAuth {
	async fn to_imap_oauth2<'a>(&'a mut self, email: &'a str) -> Result<ImapOAuth2<'a>> {
		Ok(ImapOAuth2 {
			email,
			token: self.access_token().await?,
		})
	}
}
