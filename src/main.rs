use anyhow::Result;
use news_reader::NewsReader;
use teloxide::Bot;

#[tokio::main]
async fn main() -> Result<()> {
	let news_bot = Bot::new(std::env::var("NEWS_BOT_TOKEN")?);
	let mut phoronix_news = NewsReader::new(
		"phoronix",
		"https://www.phoronix.com/rss.php",
		news_bot,
		std::env::var("NEWS_CHAT_ID")?,
	);
	phoronix_news.start().await?;
	Ok(())
}
