/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use fetcher::{
	config,
	error::Error,
	error::Result,
	run_task,
	settings::{
		self, generate_google_oauth2, generate_google_password, generate_telegram,
		generate_twitter_auth,
	},
	task::Tasks,
};
use figment::{
	providers::{Format, Toml},
	Figment,
};
use futures::future::try_join_all;
use futures::StreamExt;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook_tokio::Signals;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::{select, sync::watch::Receiver};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt()
		// FIXME: properly set INFO as default log level
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.without_time()
		.init();

	tracing::info!("Starting fetcher v{}", std::env!("CARGO_PKG_VERSION"));

	// TODO: add option to send to optional global debug chat to test first
	match std::env::args().nth(1).as_deref() {
		Some("--gen-secret-google-oauth2") => generate_google_oauth2().await?,
		Some("--gen-secret-google-password") => generate_google_password()?,
		Some("--gen-secret-telegram") => generate_telegram()?,
		Some("--gen-secret-twitter") => generate_twitter_auth()?,
		None => run().await?,
		Some(_) => panic!("error"),
	};

	Ok(())
}

async fn run() -> Result<()> {
	let tasks = settings::config::tasks()?
		.into_iter()
		.map(|(contents, path)| {
			tracing::debug!("Found task: {path:?}");
			let templates: config::Templates = Figment::new()
				.merge(Toml::string(&contents))
				.extract()
				.map_err(|e| Error::InvalidConfig(e, path.clone()))?;

			let mut conf = Figment::new();

			if let Some(templates) = templates.templates {
				for tmpl_path in templates {
					let (tmpl, tmpl_full_path) = settings::config::template(&tmpl_path)?;

					tracing::debug!("Using template: {}", tmpl_full_path.display());

					conf = conf.merge(Toml::string(&tmpl));
				}
			}

			let task: config::Task = conf
				.merge(Toml::string(&contents))
				.extract()
				.map_err(|e| Error::InvalidConfig(e, path.clone()))?;

			Ok((
				path.file_stem()
					.expect("Somehow the config file found before wasn't an actual config file after all...")
					.to_str()
					.expect("Config file name isn't a valid unicode")
					.to_string(),
				task.parse(&path)?,
			))
		})
		.collect::<Result<Tasks>>()?;

	if tasks.is_empty() {
		tracing::warn!("No tasks provided");
		return Ok(());
	}

	let (shutdown_tx, shutdown_rx) = watch::channel(());

	let sig = Signals::new(TERM_SIGNALS).expect("Error registering signals");
	let sig_handle = sig.handle();

	let sig_term_now = Arc::new(AtomicBool::new(false));
	for s in TERM_SIGNALS {
		use signal_hook::flag;

		flag::register_conditional_shutdown(
			*s,
			1, /* exit status */
			Arc::clone(&sig_term_now),
		)
		.expect("Error registering signal handler");

		flag::register(*s, Arc::clone(&sig_term_now)).expect("Error registering signal handler");
	}

	let sig_task = tokio::spawn(async move {
		let mut sig = sig.fuse();

		while sig.next().await.is_some() {
			shutdown_tx
				.send(())
				.expect("Error broadcasting signal to tasks");
		}

		Ok::<(), Error>(())
	});

	run_tasks(tasks, shutdown_rx).await?;

	sig_handle.close(); // TODO: figure out wtf this is and why
	sig_task
		.await
		.expect("Error shutting down of signal handler")?;
	Ok(())
}

async fn run_tasks(tasks: Tasks, shutdown_rx: Receiver<()>) -> Result<()> {
	let mut futs = Vec::new();
	for (name, mut t) in tasks {
		if let Some(disabled) = t.disabled {
			if disabled {
				continue;
			}
		}

		let mut shutdown_rx = shutdown_rx.clone();

		// TODO: create a tracing span for each task with task name param
		let fut = tokio::spawn(async move {
			select! {
				r = run_task(&name, &mut t) => r?,
				_ = shutdown_rx.changed() => {
					tracing::info!("{name}: shutting down...");
				},
			}

			#[allow(unreachable_code)]
			Ok::<(), Error>(())
		});

		futs.push(flatten_task(fut));
	}

	try_join_all(futs).await?;
	Ok(())
}

async fn flatten_task<T>(h: JoinHandle<Result<T>>) -> Result<T> {
	match h.await {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(err)) => Err(err),
		e => e.unwrap(), // unwrap NOTE: crash (for now) if there was an error joining the thread
	}
}
