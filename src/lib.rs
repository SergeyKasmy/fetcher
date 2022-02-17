/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: more tests

pub mod auth;
pub mod config;
pub mod error;
pub mod settings;
pub mod sink;
pub mod source;
pub mod task;

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
use task::Tasks;
use tokio::select;
use tokio::sync::watch;
use tokio::time::sleep;

use crate::error::Error;
use crate::settings::last_read_id;
use crate::settings::save_last_read_id;

#[tracing::instrument(skip_all)]
pub async fn run() -> Result<()> {
	let tasks: Tasks = toml::from_str(
		&settings::config().unwrap(), // FIXME: may crash when config.toml doesn't exist
	)
	.map_err(Error::InvalidConfig)?;

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

	let mut futs = Vec::new();
	for (name, mut t) in tasks.0 {
		if let Some(disabled) = t.disabled {
			if disabled {
				continue;
			}
		}

		let mut shutdown_rx = shutdown_rx.clone();

		let fut = tokio::spawn(async move {
			select! {
				_ = async {
					loop {
						tracing::debug!("Re-fetching {name}");
						let last_read_id = last_read_id(&name)?;

						for r in t.source.get(last_read_id).await? {
							t.sink.send(r.msg).await?;

							if let Some(id) = r.id {
								save_last_read_id(&name, id)?;
							}
						}

						sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
					}

					#[allow(unreachable_code)]
					Ok::<(), Error>(())
				} => (),
				_ = shutdown_rx.changed() => {
					tracing::info!("Shutdown signal received");
				},
			}
		});

		futs.push(fut);
	}

	// TODO: handle non critical errors, e.g. SourceFetch error
	join_all(futs).await;

	sig_handle.close(); // TODO: figure out wtf this is and why
	sig_task
		.await
		.context("Error shutting down of signal handler")??;
	Ok(())
}
