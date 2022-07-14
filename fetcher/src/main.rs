/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: proper argument parser. Something like clap or argh or something

mod settings;

use fetcher_core::{
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
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::{select, sync::watch::Receiver};
use tracing::Instrument;

fn main() -> color_eyre::Result<()> {
	{
		use tracing_subscriber::fmt::time::OffsetTime;
		use tracing_subscriber::layer::SubscriberExt;
		use tracing_subscriber::EnvFilter;
		use tracing_subscriber::Layer;

		let env_filter = EnvFilter::try_from_env("FETCHER_LOG")
			.unwrap_or_else(|_| EnvFilter::from("fetcher=info"));
		let stdout = tracing_subscriber::fmt::layer()
			.pretty()
			// hide source code/debug info on release builds
			.with_file(cfg!(debug_assertions))
			.with_line_number(cfg!(debug_assertions))
			.with_timer(OffsetTime::local_rfc_3339().expect("could not get local time offset"));

		// enable journald logging only on release to avoid log spam on dev machines
		let journald = if !cfg!(debug_assertions) {
			tracing_journald::layer().ok()
		} else {
			None
		};

		let subscriber = tracing_subscriber::registry()
			.with(journald.with_filter(tracing_subscriber::filter::LevelFilter::INFO))
			.with(stdout.with_filter(env_filter));
		tracing::subscriber::set_global_default(subscriber).unwrap();
	}

	color_eyre::install()?;

	async_main()
}

#[tokio::main]
async fn async_main() -> color_eyre::Result<()> {
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

		Some("--once") => run(true).await?,
		None => run(false).await?,
		Some(_) => panic!("error"),
	};

	Ok(())
}

async fn run(once: bool) -> Result<(), Error> {
	let tasks = settings::config::tasks::get_all(&DataSettings {
		twitter_auth: settings::data::twitter().await?,
		google_oauth2: settings::data::google_oauth2().await?,
		email_password: settings::data::google_password().await?,
		telegram: settings::data::telegram().await?,
		read_filter: Box::new(
			|name: String,
			 default: Option<fetcher_core::read_filter::Kind>|
			 -> Pin<
				Box<
					dyn Future<
						Output = Result<
							Option<fetcher_core::read_filter::ReadFilter>,
							fetcher_core::error::config::Error,
						>,
					>,
				>,
			> { Box::pin(async move { settings::read_filter::get(&name, default).await }) },
		),
	})
	.await?;

	if tasks.is_empty() {
		tracing::info!("No enabled tasks provided");
		return Ok(());
	}

	tracing::info!(
		"Found {num} enabled tasks: {names:?}",
		num = tasks.len(),
		names = tasks
			.iter()
			.map(|(name, _)| name.as_str())
			.collect::<Vec<_>>(),
	);

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

	run_tasks(tasks, shutdown_rx, once).await?;

	sig_handle.close(); // TODO: figure out wtf this is and why
	sig_task
		.await
		.expect("Error shutting down of signal handler")?;
	Ok(())
}

async fn run_tasks(tasks: Tasks, shutdown_rx: Receiver<()>, once: bool) -> Result<(), Error> {
	let mut running_tasks = Vec::new();
	for (name, mut t) in tasks {
		let name2 = name.clone(); // TODO: ehhh. Is there a way to avoid cloning?
		let mut shutdown_rx = shutdown_rx.clone();

		let fut = tokio::spawn(
			async move {
				let res: Result<(), Error> = select! {
					res = async {
						loop {
							run_task(&mut t).await?;

							if once {
								break;
							}

							tracing::debug!("Sleeping for {time}m", time = t.refresh);
							sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
						}

						Ok(())
					} => res,
					_ = shutdown_rx.changed() => Ok(()),
				};

				if let Err(e) = &res {
					// let err_str = format!("{:?}", color_eyre::eyre::eyre!(e));
					let err_str = format!("{:?}", e); // TODO: make it pretier like eyre
					tracing::error!("{}", err_str);

					// production error reporting
					// TODO: temporary, move that to a tracing layer that sends all WARN and higher logs automatically
					if !cfg!(debug_assertions) {
						if let Ok(admin_chat_id) = std::env::var("FETCHER_LOG_ADMIN_CHAT_ID") {
							use fetcher_core::sink::telegram::LinkLocation;
							use fetcher_core::sink::Message;
							use fetcher_core::sink::Telegram;

							let admin_chat_id = match admin_chat_id.parse::<i64>() {
								Ok(num) => num,
								Err(e) => {
									let s = format!(
										"Unable to send error report to the admin: FETCHER_LOG_ADMIN_CHAT_ID isn't a valid chat id ({e})"
									);
									tracing::error!(%s);
									return Err(Error::Other(s)); // TODO: this kinda sucks
								}
							};

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
								Telegram::new(bot, admin_chat_id, LinkLocation::default())
									.send(msg, Some(&name))
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
				}

				tracing::info!("Shutting down...");
				res
			}
			.instrument(tracing::info_span!("task", name = name2.as_str())),
		);

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

async fn flatten_task<T, E: std::error::Error>(h: JoinHandle<Result<T, E>>) -> Result<T, E> {
	match h.await {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(err)) => Err(err),
		e => e.unwrap(), // unwrap NOTE: crash (for now) if there was an error joining the thread
	}
}
