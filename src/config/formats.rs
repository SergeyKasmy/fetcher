use serde::{Deserialize, Serialize};

use crate::auth::GoogleAuth;
use crate::error::Result;

#[derive(Serialize, Deserialize)]
pub(crate) struct GoogleAuthCfg {
	pub client_id: String,
	pub client_secret: String,
	pub refresh_token: String,
}

impl GoogleAuthCfg {
	pub(super) async fn into_google_auth(self) -> Result<GoogleAuth> {
		GoogleAuth::new(self.client_id, self.client_secret, self.refresh_token).await
	}
}

#[derive(Serialize, Deserialize)]
pub(crate) struct TwitterAuthCfg {
	pub key: String,
	pub secret: String,
}
