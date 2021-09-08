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

		let mut shutdown_signal_rx = shutdown_signal_tx.subscribe();

		tasks.push(tokio::spawn(async move {
			loop {
				phoronix.start().await?;
				select! {
					_ = sleep(Duration::from_secs(60 * 30)) => (), // refresh every 30 mins
					_ = shutdown_signal_rx.recv() => break,
				};
			}

			println!("Phoronix News Reader quiting");
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

		let mut shutdown_signal_rx = shutdown_signal_tx.subscribe();

		tasks.push(tokio::spawn(async move {
			loop {
				apex.start().await?;
				select! {
					_ = sleep(Duration::from_secs(60 * 5)) => (), // refresh every 30 mins
					_ = shutdown_signal_rx.recv() => break,
				};
			}

			println!("Apex News Reader quiting");
			Ok::<(), anyhow::Error>(())
		}));
	}

	let signals = Signals::new(&[SignalTypes::SIGINT, SignalTypes::SIGTERM])?;
	let signals_handle = signals.handle();

	tokio::spawn(async move {
		let mut signals = signals.fuse();
		while let Some(signal) = signals.next().await {
			println!("Received a signal: {}", signal);
			shutdown_signal_tx.send(true)?;
		}

		println!("Signal receiever quitting");
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
