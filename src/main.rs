use anyhow::Context;
use anyhow::Result;
use fetcher::{config::Config, settings};

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().without_time().init();
	// tracing_log::LogTracer::init().unwrap();

	match std::env::args().nth(1).as_deref() {
		Some("--generate") => {
			fetcher::settings::generate_google_oauth2_token(
				std::env!("CLIENT_ID"),
				std::env!("CLIENT_SECRET"),
			)
			.await?;
		}
		Some(_) | None => {
			let conf = settings::get_config().context("unable to get config")?;
			let parsed = Config::parse(&conf)
				.await
				.context("unable to parse config")?;
			fetcher::run(parsed).await?;
		}
	}

	Ok(())
}
