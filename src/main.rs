use anyhow::Result;
use news_reader::NewsReader;
use teloxide::Bot;

#[tokio::main]
async fn main() -> Result<()> {
	let news_bot = Bot::new(std::env::var("BOT_TOKEN")?);
	let mut phoronix_news = NewsReader::new(
		"phoronix".to_string(),
		"https://www.phoronix.com/rss.php".to_string(),
		news_bot,
		std::env::var("CHAT_ID")?,
	);
	phoronix_news.start().await?;
	Ok(())
}
