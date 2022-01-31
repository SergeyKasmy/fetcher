use fetcher::config::Config;
use fetcher::settings;

//TODO: gracefully end execution instead of unwrapping like a monkey
#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();
	// tracing_log::LogTracer::init().unwrap();

	let parsed = Config::parse(&settings::get_config().unwrap()).await.unwrap();
	fetcher::run(parsed).await.unwrap();
}
