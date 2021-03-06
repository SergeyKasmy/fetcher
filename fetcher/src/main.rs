/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: proper argument parser. Something like clap or argh or something

mod config;
mod error;
mod settings;

use color_eyre::{eyre::eyre, Report, Result};
use futures::{future::join_all, StreamExt};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook_tokio::Signals;
use std::{
	future::Future,
	pin::Pin,
	sync::{atomic::AtomicBool, Arc},
	time::Duration,
};
use tokio::{
	select,
	sync::watch::{self, Receiver},
	task::JoinHandle,
	time::sleep,
};
use tracing::Instrument;

use crate::config::DataSettings;
use fetcher_core::{
	read_filter::Kind as ReadFilterKind,
	task::{Task, Tasks},
};

fn main() -> Result<()> {
	set_up_logging()?;
	async_main()
}

fn set_up_logging() -> Result<()> {
	use tracing_subscriber::fmt::time::OffsetTime;
	use tracing_subscriber::layer::SubscriberExt;
	use tracing_subscriber::EnvFilter;
	use tracing_subscriber::Layer;

	let env_filter =
		EnvFilter::try_from_env("FETCHER_LOG").unwrap_or_else(|_| EnvFilter::from("fetcher=info"));
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

	color_eyre::install()?;
	Ok(())
}

#[tokio::main]
async fn async_main() -> Result<()> {
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

async fn run(once: bool) -> Result<()> {
	let read_filter_getter =
		|name: String, default: Option<ReadFilterKind>| -> Pin<Box<dyn Future<Output = _>>> {
			Box::pin(async move { settings::read_filter::get(&name, default).await })
		};

	let data_settings = DataSettings {
		twitter_auth: settings::data::twitter().await?,
		google_oauth2: settings::data::google_oauth2().await?,
		email_password: settings::data::google_password().await?,
		telegram: settings::data::telegram().await?,
		read_filter: Box::new(read_filter_getter),
	};

	let tasks = settings::config::tasks::get_all(&data_settings).await?;

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

		Ok::<(), Report>(())
	});

	run_tasks(tasks, shutdown_rx, once).await?;

	sig_handle.close(); // TODO: figure out wtf this is and why
	sig_task
		.await
		.expect("Error shutting down of signal handler")?;
	Ok(())
}

async fn run_tasks(tasks: Tasks, shutdown_rx: Receiver<()>, once: bool) -> Result<()> {
	let mut running_tasks = Vec::new();
	for (name, mut t) in tasks {
		let name2 = name.clone();
		let mut shutdown_rx = shutdown_rx.clone();

		let task_handle = tokio::spawn(
			async move {
				let res = select! {
					r = task_loop(&mut t, once) => r,
					_ = shutdown_rx.changed() => Ok(()),
				};

				if let Err(err) = &res {
					let err_str = format!("{:?}", err);
					tracing::error!("{err_str}");

					// production error reporting
					if !cfg!(debug_assertions) {
						if let Err(e) = report_error(&name, &err_str).await {
							tracing::error!("Unable to send error report to the admin: {e:?}",);
						}
					}
				}

				tracing::info!("Shutting down...");
				res
			}
			.instrument(tracing::info_span!("task", name = name2.as_str())),
		);

		running_tasks.push(flatten_task_result(task_handle));
	}

	// return the first error
	let mut first_err = None;
	for res in join_all(running_tasks).await {
		if let Err(e) = res {
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

async fn task_loop(t: &mut Task, once: bool) -> Result<()> {
	loop {
		fetcher_core::run_task(t).await?;

		if once {
			break;
		}

		tracing::debug!("Sleeping for {time}m", time = t.refresh);
		sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
	}

	Ok(())
}

// TODO: move that to a tracing layer that sends all WARN and higher logs automatically
async fn report_error(task_name: &str, err: &str) -> color_eyre::Result<()> {
	use fetcher_core::sink::telegram::LinkLocation;
	use fetcher_core::sink::Message;
	use fetcher_core::sink::Telegram;

	let admin_chat_id = match std::env::var("FETCHER_ADMIN_CHAT_ID")?.parse::<i64>() {
		Ok(num) => num,
		Err(e) => {
			return Err(eyre!("FETCHER_ADMIN_CHAT_ID isn't a valid chat id ({e})"));
		}
	};
	let bot = match settings::data::telegram().await? {
		Some(b) => b,
		None => {
			return Err(eyre!("Telegram bot token not provided"));
		}
	};
	let msg = Message {
		body: err.to_owned(),
		..Default::default()
	};
	Telegram::new(bot, admin_chat_id, LinkLocation::default())
		.send(msg, Some(task_name))
		.await
		.map_err(fetcher_core::error::Error::Sink)?;

	Ok(())
}

async fn flatten_task_result<T>(h: JoinHandle<Result<T>>) -> Result<T> {
	match h.await {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(err)) => Err(err),
		e => e.unwrap(), // unwrap NOTE: crash if there was an error joining the thread
	}
}
