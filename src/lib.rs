pub mod config;
pub mod error;
pub mod settings;
pub mod sink;
pub mod source;


use futures::StreamExt;
use futures::future::join_all;
use signal_hook::consts as SignalTypes;
use signal_hook_tokio::Signals;
use std::time::Duration;
use tokio::select;
use tokio::sync::watch;
use tokio::time::sleep;

use crate::config::Config;
use crate::error::Error;
use crate::error::Result;
use crate::settings::last_read_id;
use crate::settings::save_last_read_id;

#[tracing::instrument]
pub async fn run(configs: Vec<Config>) -> Result<()> {
	let (shutdown_tx, shutdown_rx) = watch::channel(false);

	tokio::spawn(async move {
		let signals = Signals::new(&[SignalTypes::SIGINT, SignalTypes::SIGTERM]).unwrap();
		let signals_handle = signals.handle();

		let mut signals = signals.fuse();

		while signals.next().await.is_some() {
			shutdown_tx.send(true).unwrap();
			signals_handle.close();	// TODO: figure out wtf this is and why
		}
	});

	let mut tasks = Vec::new();
	for mut c in configs {
		let mut shutdown_rx = shutdown_rx.clone();

		let task = tokio::spawn(async move {
			select! {
				_ = async {
					loop {
						let last_read_id = last_read_id(&c.name)?;

						for r in c.source.get(last_read_id).await? {
							c.sink.send(r.msg).await?;

							if let Some(id) = r.id {
								save_last_read_id(&c.name, id)?;
							}
						}

						sleep(Duration::from_secs(c.refresh * 60 /* secs in a min */)).await;
					}

					#[allow(unreachable_code)]
					Ok::<(), Error>(())
				} => (),
				_ = shutdown_rx.changed() => {
					tracing::info!("Shutdown signal received");
				},
			}
		});

		tasks.push(task);
	}

	// TODO: handle non critical errors, e.g. SourceFetch error
	join_all(tasks).await;
	Ok(())
}
