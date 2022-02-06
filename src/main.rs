use anyhow::Context;
use anyhow::Result;
use fetcher::{config::Config, settings};

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().without_time().init();
	// tracing_log::LogTracer::init().unwrap();

	match std::env::args().nth(1).as_deref() {
		Some("--gen-secret-google") => {
			fetcher::settings::generate_google_oauth2().await?;
		}
		Some("--gen-secret-twitter") => {
			fetcher::settings::generate_twiiter_auth()?;
		}
		Some("--gen-secret-telegram") => {
			// fetcher::settings::generate_google_oauth2_token().await?;
			todo!()
		}
		None => {
			let conf = settings::get_config().context("unable to get config")?;
			let parsed = Config::parse(&conf)
				.await
				.context("unable to parse config")?;
			fetcher::run(parsed).await?;
		}
		Some(_) => panic!("error"),
	}

	Ok(())
}
