pub mod config;
pub mod error;
pub mod settings;
pub mod sink;
pub mod source;

// TODO: mb using anyhow in lib code isn't a good idea?
use anyhow::Context;
use anyhow::Result;
use futures::future::join_all;
use futures::StreamExt;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook_tokio::Signals;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::watch;
use tokio::time::sleep;

use crate::config::Config;
use crate::error::Error;
use crate::settings::last_read_id;
use crate::settings::save_last_read_id;

// TODO: more tests
#[tracing::instrument(skip_all)]
pub async fn run(configs: Vec<Config>) -> Result<()> {
	let (shutdown_tx, shutdown_rx) = watch::channel(false);

	let sig = Signals::new(TERM_SIGNALS).context("Error registering signals")?;
	let sig_handle = sig.handle();

	let sig_term_now = Arc::new(AtomicBool::new(false));
	for s in TERM_SIGNALS {
		use signal_hook::flag;

		flag::register_conditional_shutdown(
			*s,
			1, /* exit status */
			Arc::clone(&sig_term_now),
		)
		.context("Error registering signal handler")?;

		flag::register(*s, Arc::clone(&sig_term_now))
			.context("Error registering signal handler")?;
	}

	let sig_task = tokio::spawn(async move {
		let mut sig = sig.fuse();

		while sig.next().await.is_some() {
			shutdown_tx
				.send(true)
				.context("Error broadcasting signal to tasks")?;
		}

		Ok::<(), anyhow::Error>(())
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

	// FIXME: handle non critical errors, e.g. SourceFetch error
	join_all(tasks).await;

	sig_handle.close(); // TODO: figure out wtf this is and why
	sig_task
		.await
		.context("Error shutting down of signal handler")??;
	Ok(())
}
