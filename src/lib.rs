pub mod config;
pub mod error;
pub(crate) mod guid;
pub mod providers;
pub mod telegram;

use config::Config;
use error::Error;
use error::Result;
use futures::future::select_all;
use futures::StreamExt;
use signal_hook::consts as SignalTypes;
use signal_hook_tokio::Signals;
use std::time::Duration;
use tokio::{select, sync::broadcast, time::sleep};

#[tracing::instrument]
pub async fn run(configs: Vec<Config>) -> Result<()> {
	let (shutdown_sig_tx, _) = broadcast::channel(1);
	let mut tasks = Vec::new();

	// TODO: add sleep time to configs
	for mut c in configs {
		let mut rx = shutdown_sig_tx.subscribe();
		let task = tokio::spawn(async move {
			loop {
				for m in c.source.get().await?.into_iter() {
					c.sink.send(m).await?;
				}
				select! {
					_ = async {
						tracing::info!("Refreshing {name} in {refresh}m", name = c.name, refresh = c.refresh);
						sleep(Duration::from_secs(c.refresh * 60 /* seconds in a minute */)).await;
					} => (),
					_ = rx.recv() => break,
				}
			}

			Ok::<(), Error>(())
		});
		tasks.push(task);
	}

	let signals = Signals::new(&[SignalTypes::SIGINT, SignalTypes::SIGTERM]).unwrap();
	let signals_handle = signals.handle();

	tokio::spawn(async move {
		let mut signals = signals.fuse();
		while signals.next().await.is_some() {
			shutdown_sig_tx.send(()).unwrap();
		}

		Ok::<(), error::Error>(())
	});

	loop {
		let finished_task = select_all(tasks).await;
		match finished_task.0.unwrap() {
			// TODO: rerun the task after an error instead of ignoring it outright
			Ok(_) | Err(Error::Get { .. }) => {
				if !finished_task.2.is_empty() {
					tasks = finished_task.2;
				} else {
					break;
				}
			}
			Err(e) => return Err(e),
		}
	}

	signals_handle.close();
	Ok(())
}
