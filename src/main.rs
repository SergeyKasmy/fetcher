use anyhow::Context;
use anyhow::Result;
use fetcher::{config::Config, settings};

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().without_time().init();
	// tracing_log::LogTracer::init().unwrap();

	let conf = settings::get_config().context("unable to get config")?;
	let parsed = Config::parse(&conf)
		.await
		.context("unable to parse config")?;
	fetcher::run(parsed).await?;

	Ok(())
}
