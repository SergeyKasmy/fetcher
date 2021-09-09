use anyhow::anyhow;
use anyhow::Result;
use futures::future::select_all;
use futures::stream::StreamExt;
use news_reader::{RssNewsReader, TwitterNewsReader};
use signal_hook::consts as SignalTypes;
use signal_hook_tokio::Signals;
use std::{env, time::Duration};
use teloxide::Bot;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

macro_rules! create_task {
    ($run: stmt, $sig: expr, $dur: expr) => {
        tokio::spawn(async move {
            loop {
                $run
				select! {
					_ = sleep(Duration::from_secs(60 * $dur)) => (), // refresh every $dur mins
					_ = $sig.recv() => break,
				};
            }

			Ok::<(), anyhow::Error>(())
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let (shutdown_signal_tx, _) = broadcast::channel(1);

	let mut tasks = Vec::new();
	let news_bot = Bot::new(env::var("NEWS_BOT_TOKEN")?);

	{
		let mut phoronix = RssNewsReader::new(
			"phoronix",
			"https://www.phoronix.com/rss.php",
			news_bot.clone(),
			env::var("PHORONIX_CHAT_ID")?,
		);

		let mut rx = shutdown_signal_tx.subscribe();
		tasks.push(create_task!(phoronix.start().await?, rx, 30));
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

		let mut rx = shutdown_signal_tx.subscribe();
		tasks.push(create_task!(apex.start().await?, rx, 5));
	}

	let signals = Signals::new(&[SignalTypes::SIGINT, SignalTypes::SIGTERM])?;
	let signals_handle = signals.handle();

	tokio::spawn(async move {
		let mut signals = signals.fuse();
		while signals.next().await.is_some() {
			shutdown_signal_tx.send(())?;
		}

		Ok::<(), anyhow::Error>(())
	});

	loop {
		let finished_task = select_all(tasks).await;
		match finished_task.0? {
			Ok(_) => {
				if !finished_task.2.is_empty() {
					tasks = finished_task.2;
				} else {
					break;
				}
			}
			Err(e) => return Err(anyhow!(e)),
		}
	}

	signals_handle.close();
	Ok(())
}
