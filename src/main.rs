use anyhow::Result;
use news_reader::{RssNewsReader, TwitterNewsReader};
use std::env;
use teloxide::Bot;

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let news_bot = Bot::new(env::var("NEWS_BOT_TOKEN")?);

	RssNewsReader::new(
		"phoronix",
		"https://www.phoronix.com/rss.php",
		news_bot.clone(),
		env::var("PHORONIX_CHAT_ID")?,
	)
	.start()
	.await?;

	TwitterNewsReader::new(
		"apex",
		"ApexLegends",
		"@Respawn",
		env::var("TWITTER_API_KEY")?,
		env::var("TWITTER_API_KEY_SECRET")?,
		Some(&["@playapex", "update"]),
		news_bot.clone(),
		env::var("GAMING_CHAT_ID")?,
	)
	.await?
	.start()
	.await?;

	Ok(())
}
