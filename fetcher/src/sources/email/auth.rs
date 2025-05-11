/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
	StaticStr,
	auth::google::{Google as GoogleAuth, GoogleOAuth2Error as GoogleAuthError},
};

/// Authentication type for IMAP
pub enum Auth {
	#[expect(clippy::doc_markdown, reason = "false positive")]
	/// Google OAuth2 with full access to Gmail
	GmailOAuth2(GoogleAuth),
	/// An insecure pure text password
	Password(StaticStr),
}

pub(super) struct ImapOAuth2<'a> {
	email: &'a str,
	token: &'a str,
}

impl async_imap::Authenticator for ImapOAuth2<'_> {
	type Response = String;

	fn process(&mut self, _challenge: &[u8]) -> Self::Response {
		format!("user={}\x01auth=Bearer {}\x01\x01", self.email, self.token)
	}
}

pub(super) trait GoogleAuthExt {
	async fn as_imap_oauth2<'a>(
		&'a mut self,
		email: &'a str,
	) -> Result<ImapOAuth2<'a>, GoogleAuthError>;
}

impl GoogleAuthExt for GoogleAuth {
	async fn as_imap_oauth2<'a>(
		&'a mut self,
		email: &'a str,
	) -> Result<ImapOAuth2<'a>, GoogleAuthError> {
		Ok(ImapOAuth2 {
			email,
			token: self.access_token().await?,
		})
	}
}
