use news_reader::config::Config;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();
	// tracing_log::LogTracer::init().unwrap();

	let conf_path = xdg::BaseDirectories::with_prefix("news-reader").unwrap()
		.place_config_file("config.toml").unwrap();
	let conf = std::fs::read_to_string(&conf_path)
		.unwrap_or_else(|_| panic!("{:?} doesn't exist", conf_path));

	let parsed = Config::parse(&conf).await.unwrap();
	news_reader::run(parsed).await.unwrap();
}
