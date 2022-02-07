/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use anyhow::Result;
use fetcher::{
	run,
	settings::{
		generate_google_oauth2, generate_google_password, generate_telegram, generate_twitter_auth,
	},
};

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().without_time().init();
	// tracing_log::LogTracer::init().unwrap();

	match std::env::args().nth(1).as_deref() {
		Some("--gen-secret-google-oauth2") => generate_google_oauth2().await?,
		Some("--gen-secret-google-password") => generate_google_password()?,
		Some("--gen-secret-telegram") => generate_telegram()?,
		Some("--gen-secret-twitter") => generate_twitter_auth()?,
		None => run().await?,
		Some(_) => panic!("error"),
	};

	Ok(())
}
