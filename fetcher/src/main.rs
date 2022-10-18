/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]

pub mod args;
pub mod settings;

use self::settings::context::StaticContext as Context;
use crate::args::{Args, Setting};
use fetcher_config::tasks::{ParsedTask, ParsedTasks};
use fetcher_core::error::{Error, ErrorChainExt};

use color_eyre::{eyre::eyre, Report, Result};
use futures::{future::try_join_all, StreamExt};
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
	async_main()?;

	Ok(())
}

fn set_up_logging() -> Result<()> {
	use tracing_subscriber::{
		filter::LevelFilter, fmt::time::OffsetTime, layer::SubscriberExt, EnvFilter, Layer,
	};

	let env_filter = EnvFilter::try_from_env("FETCHER_LOG")
		.unwrap_or_else(|_| EnvFilter::from("fetcher=info,fetcher_core=info"));

	let stdout = tracing_subscriber::fmt::layer()
		.pretty()
		// hide source code/debug info on release builds
		// .with_file(cfg!(debug_assertions))
		// .with_line_number(cfg!(debug_assertions))
		.with_timer(OffsetTime::local_rfc_3339().expect("could not get local time offset"));

	// enable journald logging only on release to avoid log spam on dev machines
	let journald = if cfg!(debug_assertions) {
		None
	} else {
		tracing_journald::layer().ok()
	};

	let subscriber = tracing_subscriber::registry()
		.with(journald.with_filter(LevelFilter::INFO))
		.with(stdout.with_filter(env_filter));
	tracing::subscriber::set_global_default(subscriber).unwrap();

	color_eyre::install()?;
	Ok(())
}

#[tokio::main]
async fn async_main() -> Result<()> {
	let args: Args = argh::from_env();
	let context: Context = {
		let data_path = match args.data_path {
			Some(p) => p,
			None => settings::data::default_data_path()?,
		};
		let conf_paths = match args.config_path {
			Some(p) => vec![p],
			None => settings::config::default_cfg_dirs()?,
		};

		Box::leak(Box::new(crate::settings::context::Context {
			data_path,
			conf_paths,
		}))
	};

	match args.subcommand {
		args::TopLvlSubcommand::Run(arg) => {
			run(
				if arg.tasks.is_empty() {
					None
				} else {
					Some(arg.tasks)
				},
				arg.once,
				context,
			)
			.await?;
		}
		args::TopLvlSubcommand::Save(save) => match save.setting {
			Setting::GoogleOAuth2 => settings::data::google_oauth2::prompt(context).await?,
			Setting::EmailPassword => settings::data::email_password::prompt(context)?,
			Setting::Telegram => settings::data::telegram::prompt(context)?,
			Setting::Twitter => settings::data::twitter::prompt(context)?,
		},
	}

	Ok(())
}

/// Run once or loop?
/// If specified, run only tasks in `run_by_name`
async fn run(run_by_name: Option<Vec<String>>, once: bool, cx: Context) -> Result<()> {
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

	let run_by_name_is_some = run_by_name.is_some();
	let tasks = get_all_tasks(run_by_name, cx).await?;

	if run_by_name_is_some {
		if tasks.is_empty() {
			tracing::info!("No enabled tasks found for the provided query");

			let all_tasks = get_all_tasks(None, cx).await?;
			tracing::info!(
				"All available enabled tasks: {:?}",
				all_tasks.keys().collect::<Vec<_>>()
			);

			return Ok(());
		}

		tracing::info!(
			"Found {} enabled tasks for the provided query: {:?}",
			tasks.len(),
			tasks.keys().collect::<Vec<_>>()
		);
	} else {
		if tasks.is_empty() {
			tracing::info!("No enabled tasks found");
			return Ok(());
		}

		tracing::info!(
			"Found {} enabled tasks: {:?}",
			tasks.len(),
			tasks.keys().collect::<Vec<_>>()
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

		Ok::<(), Report>(())
	});

	run_tasks(tasks, shutdown_rx, once, cx).await?;

	sig_handle.close(); // TODO: figure out wtf this is and why
	sig_task
		.await
		.expect("Error shutting down of signal handler")?;
	Ok(())
}

async fn get_all_tasks(run_by_name: Option<Vec<String>>, cx: Context) -> Result<ParsedTasks> {
	tokio::task::spawn_blocking(move || {
		settings::config::tasks::get_all(
			run_by_name
				.as_ref()
				.map(|s| s.iter().map(String::as_str).collect::<Vec<_>>())
				.as_deref(),
			cx,
		)
	})
	.await
	.expect("Thread crashed")
}

async fn run_tasks(
	tasks: ParsedTasks,
	shutdown_rx: Receiver<()>,
	once: bool,
	context: Context,
) -> Result<()> {
	let mut running_tasks = Vec::new();
	for (name, mut t) in tasks {
		let name2 = name.clone();
		let mut shutdown_rx = shutdown_rx.clone();

		let task_handle = tokio::spawn(
			async move {
				tracing::trace!("Task {} contents: {:#?}", name, t);

				let res = select! {
					r = task_loop(&mut t, &name, once) => r,
					_ = shutdown_rx.changed() => Ok(()),
				};

				// production error reporting
				if !cfg!(debug_assertions) {
					if let Err(err) = &res {
						if let Err(e) = report_error(&name, &format!("{:#}", err), &context).await {
							tracing::error!("Unable to send error report to the admin: {e:?}",);
						}
					}
				}
				tracing::info!("Task {name} shut down...");

				res
			}
			.instrument(tracing::info_span!("task", name = name2.as_str())),
		);

		running_tasks.push(flatten_task_result(task_handle));
	}

	try_join_all(running_tasks).await?;
	Ok(())
}

async fn task_loop(t: &mut ParsedTask, task_name: &str, once: bool) -> Result<()> {
	// exit with an error if there were too many consecutive transform errors
	let transform_err_max_count: u32 = if once { 0 } else { 255 };
	// number of consecutive(!!!) transform errors.
	// we tolerate a pretty big amount for various reasons (being rate limited, server error, etc) but not infinite
	let mut transform_err_count = 0;

	loop {
		// return critical errors and just log non critical ones
		match fetcher_core::run_task(&mut t.inner).await {
			Ok(()) => {
				transform_err_count = 0;
			}
			Err(Error::Transform(transform_err)) => {
				settings::state::log_transform_err(&transform_err, task_name).await?;

				if transform_err_count == transform_err_max_count {
					return Err(Error::Transform(transform_err).into());
				}

				tracing::error!(
					"Transform error ({} out of {} max allowed):\n{}",
					transform_err_count + 1, // +1 cause we are counting from 0 but it'd be strange to show "Error (0 out of 255)" to users
					transform_err_max_count + 1,
					transform_err.display_chain()
				);

				// sleep in exponention amount of minutes, begginning with 2^0 = 1 minute
				let sleep_dur = 2u64.saturating_pow(transform_err_count);
				tracing::info!("Pausing task {task_name} for {sleep_dur}m");

				sleep(Duration::from_secs(sleep_dur * 60 /* secs in a min*/)).await;
				transform_err_count += 1;

				continue;
			}
			Err(e) => {
				if let Some(network_err) = e.is_connection_error() {
					tracing::warn!("Network error: {}", network_err.display_chain());
				} else {
					return Err(e.into());
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
async fn report_error(task_name: &str, err: &str, context: &Context) -> Result<()> {
	use fetcher_core::sink::{telegram::LinkLocation, Message, Telegram};

	let admin_chat_id = match std::env::var("FETCHER_TELEGRAM_ADMIN_CHAT_ID")?.parse::<i64>() {
		Ok(num) => num,
		Err(e) => {
			return Err(eyre!(
				"FETCHER_TELEGRAM_ADMIN_CHAT_ID isn't a valid chat id ({e})"
			));
		}
	};
	let bot = match settings::data::telegram::get(context)? {
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
		e => e.expect("Thread panicked"),
	}
}
