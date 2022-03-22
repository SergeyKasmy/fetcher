/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// 22.03.22 03:30 CONTINUE: source templates
// TODO: proper argument parser. Something like clap or argh or something

mod settings;

use fetcher::error::Result;
use fetcher::{
	config::{self, DataSettings},
	error::Error,
	run_task,
	task::Tasks,
};
use futures::future::join_all;
use futures::StreamExt;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook_tokio::Signals;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::{select, sync::watch::Receiver};
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	tracing_subscriber::fmt()
		.pretty()
		.with_env_filter(
			EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::from("fetcher=info")), // TODO: that doesn't look right. Isn't there a better way to use info by default?
		)
		// .without_time()
		.init();
	color_eyre::install()?;

	let version = if std::env!("VERGEN_GIT_BRANCH") == "main" {
		std::env!("VERGEN_GIT_SEMVER_LIGHTWEIGHT")
	} else {
		concat!(
			"v",
			std::env!("VERGEN_GIT_SEMVER_LIGHTWEIGHT"),
			"-",
			std::env!("VERGEN_GIT_SHA_SHORT"),
			" on branch ",
			std::env!("VERGEN_GIT_BRANCH")
		)
	};
	tracing::info!("Running fetcher {}", version);

	// TODO: add option to send to optional global debug chat to test first
	match std::env::args().nth(1).as_deref() {
		Some("--gen-secret-google-oauth2") => settings::data::generate_google_oauth2().await?,
		Some("--gen-secret-google-password") => settings::data::generate_google_password().await?,
		Some("--gen-secret-telegram") => settings::data::generate_telegram().await?,
		Some("--gen-secret-twitter") => settings::data::generate_twitter_auth().await?,
		None => run().await?,
		Some(_) => panic!("error"),
	};

	Ok(())
}

async fn run() -> Result<()> {
	let tasks = settings::config::tasks::get_all(&DataSettings {
		twitter_auth: settings::data::twitter().await?,
		google_oauth2: settings::data::google_oauth2().await?,
		google_password: settings::data::google_password().await?,
		telegram: settings::data::telegram().await?,
		read_filter: Box::new(
			|name: String,
			 default: Option<fetcher::read_filter::Kind>|
			 -> Pin<Box<dyn Future<Output = Result<Option<fetcher::read_filter::ReadFilter>>>>> {
				Box::pin(async move { settings::read_filter::get(&name, default).await })
			},
		),
	})
	.await?;

	if tasks.is_empty() {
		tracing::warn!("No enabled tasks provided");
		return Ok(());
	} else {
		tracing::debug!(
			"Found {num} enabled tasks: {names:?}",
			num = tasks.len(),
			names = tasks.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
		);
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
		.expect("Error registering signal handler"); // unwrap NOTE: crash if even signal handlers can't be set up

		// unwrap NOTE: crash if even signal handlers can't be set up
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
	let mut running_tasks = Vec::new();
	for mut t in tasks {
		let name = t.name.clone(); // TODO: ehhh
		let mut shutdown_rx = shutdown_rx.clone();

		let fut = tokio::spawn(async move {
			let res: Result<()> = async {
				// let mut read_filter =
				// 	settings::read_filter::get(&name, t.read_filter_kind()).await?;

				select! {
					res = run_task(&mut t) => res,
					_ = shutdown_rx.changed() => Ok(()),
				}
			}
			.instrument(tracing::info_span!("task", name = name.as_str()))
			.await;

			// production error reporting
			if let Err(e) = &res {
				if !cfg!(debug_assertions) {
					// TODO: temporary, move that to a tracing layer that sends all WARN and higher logs automatically
					use fetcher::sink::Message;
					use fetcher::sink::Telegram;

					// let err_str = format!("{:?}", color_eyre::eyre::eyre!(e));
					let err_str = format!("{:?}", e); // TODO: make it pretier like eyre
					tracing::error!("{}", err_str);
					let send_job = async {
						let bot = match settings::data::telegram().await? {
							Some(b) => b,
							None => {
								let s = "Unable to send error report to the admin: telegram bot token is not provided".to_owned();
								tracing::error!(%s);
								return Err(Error::Other(s)); // TODO: this kinda sucks
							}
						};
						let msg = Message {
							body: err_str,
							..Default::default()
						};
						Telegram::new(bot, std::env!("FETCHER_DEBUG_ADMIN_CHAT_ID").to_owned())
							.send(msg, Some(&t.name))
							.await?;
						Ok::<(), Error>(())
					};
					if let Err(e) = send_job.await {
						tracing::error!(
							"Unable to send error report to the admin: {:?}",
							// color_eyre::eyre::eyre!(e)
							e
						);
					}
				}
			}

			tracing::info!("Shutting down...");
			res
		});

		running_tasks.push(flatten_task(fut));
	}

	// print every error but return only the first
	let mut first_err = None;
	for res in join_all(running_tasks).await {
		if let Err(e) = res {
			tracing::error!("{:?}", e);
			if first_err.is_none() {
				first_err = Some(e);
			}
		}
	}

	match first_err {
		None => Ok(()),
		Some(e) => Err(e),
	}
}

async fn flatten_task<T>(h: JoinHandle<Result<T>>) -> Result<T> {
	match h.await {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(err)) => Err(err),
		e => e.unwrap(), // unwrap NOTE: crash (for now) if there was an error joining the thread
	}
}
