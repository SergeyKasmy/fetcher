use anyhow::Result;
use futures::future::select_all;
use news_reader::{RssNewsReader, TwitterNewsReader};
use std::{env, time::Duration};
use teloxide::Bot;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let mut tasks = Vec::new();

	let news_bot = Bot::new(env::var("NEWS_BOT_TOKEN")?);

	{
		let mut phoronix = RssNewsReader::new(
			"phoronix",
			"https://www.phoronix.com/rss.php",
			news_bot.clone(),
			env::var("PHORONIX_CHAT_ID")?,
		);

		tasks.push(tokio::spawn(async move {
			loop {
				phoronix.start().await?;
				sleep(Duration::from_secs(60 * 30)).await; // refresh every 30 mins
			}

			#[allow(unreachable_code)]
			Ok::<(), anyhow::Error>(())
		}));
	}

	{
		let mut apex = TwitterNewsReader::new(
			"apex",
			"ApexLegends",
			"@Respawn",
			env::var("TWITTER_API_KEY")?,
			env::var("TWITTER_API_KEY_SECRET")?,
			Some(&["@playapex", "update"]),
			news_bot.clone(),
			env::var("GAMING_CHAT_ID")?,
		)
		.await?;

		tasks.push(tokio::spawn(async move {
			loop {
				apex.start().await?;
				sleep(Duration::from_secs(60 * 5)).await; // refresh every 5 mins
			}

			#[allow(unreachable_code)]
			Ok::<(), anyhow::Error>(())
		}));
	}

	// NOTE: tasks don't (read: shoudn't) finish by themselves, only if there has occured an error.
	// Wait for the first task to finish (with an error) and propagate it
	select_all(tasks).await.0??;
	Ok(())
}
