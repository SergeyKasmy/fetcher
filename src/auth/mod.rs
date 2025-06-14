/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
	rustdoc::broken_intra_doc_links,
	reason = "it's broken when google-oauth2 feature isn't enabled"
)]
//! This module contains all external manual authentication implementations. For now it's just [`Google OAuth2`](`Google`)

#[cfg(feature = "google-oauth2")]
pub mod google;

#[cfg(feature = "google-oauth2")]
pub use google::Google;

use crate::error::Error;

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
	#[cfg(feature = "google-oauth2")]
	#[error("Google authentication error")]
	GoogleOAuth2(#[from] google::GoogleOAuth2Error),
}

impl Error for AuthError {
	fn is_network_related(&self) -> Option<&dyn Error> {
		match self {
			#[cfg(feature = "google-oauth2")]
			Self::GoogleOAuth2(e) => e.is_network_related(),

			#[allow(
				unreachable_patterns,
				reason = "reachable in some feature combinations"
			)]
			_ => None,
		}
	}
}
