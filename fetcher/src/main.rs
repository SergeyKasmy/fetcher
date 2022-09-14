/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: proper argument parser. Something like clap or argh or something
// TODO: make fetcher_config more easily replaceable

#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

pub mod args;
pub mod settings;

use crate::args::{Args, Setting};
use fetcher_config::tasks::{ParsedTask, ParsedTasks};
use fetcher_core::{error::Error, error::ErrorChainExt};

use color_eyre::{eyre::eyre, Report, Result};
use futures::{future::join_all, StreamExt};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook_tokio::Signals;
use std::{
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

fn main() -> Result<()> {
	set_up_logging()?;
	async_main()
}

fn set_up_logging() -> Result<()> {
	use tracing_subscriber::fmt::time::OffsetTime;
	use tracing_subscriber::layer::SubscriberExt;
	use tracing_subscriber::EnvFilter;
	use tracing_subscriber::Layer;

	let env_filter = EnvFilter::try_from_env("FETCHER_LOG")
		.unwrap_or_else(|_| EnvFilter::from("fetcher=info,fetcher_core=info"));
	let stdout = tracing_subscriber::fmt::layer()
		.pretty()
		// hide source code/debug info on release builds
		.with_file(cfg!(debug_assertions))
		.with_line_number(cfg!(debug_assertions))
		.with_timer(OffsetTime::local_rfc_3339().expect("could not get local time offset"));

	// enable journald logging only on release to avoid log spam on dev machines
	let journald = if cfg!(debug_assertions) {
		None
	} else {
		tracing_journald::layer().ok()
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
	let args: Args = argh::from_env();
	settings::DATA_PATH
		.set(match args.data_path {
			Some(p) => p,
			None => settings::data::default_data_path()?,
		})
		.unwrap();
	settings::CONF_PATHS
		.set(match args.config_path {
			Some(p) => vec![p],
			None => settings::config::default_cfg_dirs()?,
		})
		.unwrap();

	match args.subcommand {
		args::Subcommands::Run(arg) => run(arg.once).await?,
		args::Subcommands::Save(save) => match save.setting {
			Setting::GoogleOAuth2 => settings::data::google_oauth2::prompt().await?,
			Setting::EmailPassword => settings::data::email_password::prompt()?,
			Setting::Telegram => settings::data::telegram::prompt()?,
			Setting::Twitter => settings::data::twitter::prompt()?,
		},
	}

	Ok(())
}

async fn run(once: bool) -> Result<()> {
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

	let tasks = settings::config::tasks::get_all().await?;

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

async fn run_tasks(tasks: ParsedTasks, shutdown_rx: Receiver<()>, once: bool) -> Result<()> {
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
					let err_str = err.display_chain();
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

	// TODO: aggregate multiple errors into one using color_eyre Section trait
	match first_err {
		None => Ok(()),
		Some(e) => Err(e.into()),
	}
}

async fn task_loop(t: &mut ParsedTask, once: bool) -> Result<(), Error> {
	loop {
		match fetcher_core::run_task(&mut t.inner).await {
			Ok(()) => (),
			Err(Error::Transform(transform_err)) => {
				tracing::error!("Transform error: {}", transform_err.display_chain());
			}
			Err(e) => {
				if let Some(network_err) = e.is_connection_error() {
					tracing::warn!("Network error: {}", network_err.display_chain());
				} else {
					return Err(e);
				}
			}
		}

		if once {
			break;
		}

		tracing::debug!("Sleeping for {time}m", time = t.refresh);
		sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
	}

	Ok(())
}

// TODO: move that to a tracing layer that sends all WARN and higher logs automatically
async fn report_error(task_name: &str, err: &str) -> Result<()> {
	use fetcher_core::sink::telegram::LinkLocation;
	use fetcher_core::sink::Message;
	use fetcher_core::sink::Telegram;

	let admin_chat_id = match std::env::var("FETCHER_TELEGRAM_ADMIN_CHAT_ID")?.parse::<i64>() {
		Ok(num) => num,
		Err(e) => {
			return Err(eyre!(
				"FETCHER_TELEGRAM_ADMIN_CHAT_ID isn't a valid chat id ({e})"
			));
		}
	};
	let bot = match settings::data::telegram::get()? {
		Some(b) => b,
		None => {
			return Err(eyre!("Telegram bot token not provided"));
		}
	};
	let msg = Message {
		body: Some(err.to_owned()),
		..Default::default()
	};
	Telegram::new(bot, admin_chat_id, LinkLocation::default())
		.send(msg, Some(task_name))
		.await
		.map_err(fetcher_core::error::Error::Sink)?;

	Ok(())
}

async fn flatten_task_result<T, E>(h: JoinHandle<Result<T, E>>) -> Result<T, E> {
	match h.await {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(err)) => Err(err),
		e => e.unwrap(), // unwrap NOTE: crash if there was an error joining the thread
	}
}
