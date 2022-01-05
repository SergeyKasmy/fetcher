use news_reader::{error::Result, Config};

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	//news_reader::run(configs)
	Ok(())
}
