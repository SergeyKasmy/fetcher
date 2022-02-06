use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().without_time().init();
	// tracing_log::LogTracer::init().unwrap();

	match std::env::args().nth(1).as_deref() {
		Some("--gen-secret-google") => {
			fetcher::settings::generate_google_oauth2().await?;
		}
		Some("--gen-secret-twitter") => {
			fetcher::settings::generate_twitter_auth()?;
		}
		Some("--gen-secret-telegram") => {
			fetcher::settings::generate_telegram()?;
		}
		None => {
			fetcher::run().await?;
		}
		Some(_) => panic!("error"),
	}

	Ok(())
}
