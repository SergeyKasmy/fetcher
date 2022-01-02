use anyhow::anyhow;
use anyhow::Result;
use futures::future::select_all;
use futures::stream::StreamExt;
use news_reader::providers::email::EmailFilter;
use news_reader::{error::Error, providers::*, telegram::Telegram};
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
		let phoronix_bot = Telegram::new(news_bot.clone(), env::var("PHORONIX_CHAT_ID")?);
		let mut phoronix = Rss::new("phoronix", "https://www.phoronix.com/rss.php");

		let mut rx = shutdown_signal_tx.subscribe();
		let task = tokio::spawn(async move {
			loop {
				for m in phoronix.get_and_save().await?.into_iter() {
					phoronix_bot.send(m).await?;
				}
				select! {
					_ = sleep(Duration::from_secs(60 * 30)) => (),
					_ = rx.recv() => break,
				}
			}

			Ok::<(), Error>(())
		});
		tasks.push(task);
	}

	{
		let apex_bot = Telegram::new(news_bot.clone(), env::var("GAMING_CHAT_ID")?);
		let mut apex = Twitter::new(
			"apex",
			"ApexLegends",
			"@Respawn",
			env::var("TWITTER_API_KEY")?,
			env::var("TWITTER_API_KEY_SECRET")?,
			Some(&["@playapex"]),
		)
		.await?;

		let mut rx = shutdown_signal_tx.subscribe();
		let task = tokio::spawn(async move {
			loop {
				for m in apex.get_and_save().await?.into_iter() {
					apex_bot.send(m).await?;
				}
				select! {
					_ = sleep(Duration::from_secs(60 * 5)) => (),
					_ = rx.recv() => break,
				}
			}

			Ok::<(), Error>(())
		});
		tasks.push(task);
	}

	{
		let releases_bot = Telegram::new(news_bot.clone(), env::var("RELEASES_CHAT_ID")?);
		let mut github_releases = Email::new(
			"Github Releases",
			"imap.gmail.com",
			env::var("EMAIL")?,
			env::var("EMAIL_PASS")?,
			Some(&[
				EmailFilter::Sender("notifications@github.com"),
				EmailFilter::Subject("release"),
			]),
			false,
			Some("\r\n\r\n-- \r\n"),
		);

		let mut rx = shutdown_signal_tx.subscribe();
		let task = tokio::spawn(async move {
			loop {
				for m in github_releases.get().await?.into_iter() {
					releases_bot.send(m).await?;
				}
				select! {
					_ = sleep(Duration::from_secs(60 * 30)) => (),
					_ = rx.recv() => break,
				}
			}

			Ok::<(), Error>(())
		});
		tasks.push(task);
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
			// TODO: rerun the task after an error instead of ignoring it outright
			Ok(_) | Err(Error::Get { .. }) => {
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
