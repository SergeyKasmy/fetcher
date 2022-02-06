use anyhow::Result;
use fetcher::run;
use fetcher::settings::generate_google_oauth2;
use fetcher::settings::generate_telegram;
use fetcher::settings::generate_twitter_auth;

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().without_time().init();
	// tracing_log::LogTracer::init().unwrap();

	match std::env::args().nth(1).as_deref() {
		Some("--gen-secret-google") => generate_google_oauth2().await?,
		Some("--gen-secret-telegram") => generate_telegram()?,
		Some("--gen-secret-twitter") => generate_twitter_auth()?,
		None => run().await?,
		Some(_) => panic!("error"),
	};

	Ok(())
}
