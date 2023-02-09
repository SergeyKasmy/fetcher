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

use self::settings::{
	context::Context as OwnedContext, context::StaticContext as Context, run_mode::RunMode,
};
use crate::args::{Args, Setting};
use fetcher_config::tasks::{ParsedTask, ParsedTasks};
use fetcher_core::error::{Error, ErrorChainExt};

use color_eyre::{
	eyre::{eyre, WrapErr},
	Report, Result,
};
use futures::future::join_all;
use std::{fmt::Write, time::Duration};
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
		let log_path = match args.log_path {
			Some(p) => p,
			None => settings::log::default_log_path()?,
		};

		Box::leak(Box::new(OwnedContext {
			data_path,
			conf_paths,
			log_path,
		}))
	};

	match args.subcommand {
		args::TopLvlSubcommand::Run(arg) => {
			let mode = if arg.verify_only {
				tracing::info!("Running in \"verify only\" mode");
				RunMode::VerifyOnly
			} else if arg.mark_old_as_read {
				tracing::info!("Running in \"Mark old entries as read and leave\" move");
				RunMode::MarkOldEntriesAsRead
			} else {
				RunMode::Normal {
					once: arg.once,
					dry_run: arg.dry_run,
				}
			};

			run(
				if arg.tasks.is_empty() {
					None
				} else {
					Some(arg.tasks)
				},
				mode,
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
async fn run(run_by_name: Option<Vec<String>>, mode: RunMode, cx: Context) -> Result<()> {
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
	let (force_close_tx, mut force_close_rx) = watch::channel(());

	// signal handler
	tokio::spawn(async move {
		// graceful shutdown
		tokio::signal::ctrl_c()
			.await
			.expect("failed to setup signal handler");

		// shutdown signal recieved
		shutdown_tx
			.send(())
			.expect("failed to broadcast shutdown signal to tasks");

		tracing::info!("Press Ctrl-C again to force close");

		// force close
		tokio::signal::ctrl_c()
			.await
			.expect("failed to setup signal handler");

		force_close_tx
			.send(())
			.expect("failed to broadcast force shutdown signal");

		Ok::<(), Report>(())
	});

	// force close signal receiver
	tokio::spawn(async move {
		force_close_rx
			.changed()
			.await
			.expect("force close transmitter has been closed");

		tracing::info!("Force closing...");

		std::process::exit(1);
	});

	match mode {
		RunMode::Normal { once, dry_run } => {
			let mut tasks = tasks;

			if dry_run {
				tracing::debug!("Making all tasks dry");

				make_tasks_dry(&mut tasks).await;
			}

			run_tasks(tasks, shutdown_rx, once, cx)
				.await
				.map_err(|errs| {
					eyre!(
						"{} tasks have finished with an error: {}",
						errs.len(),
						errs.into_iter().enumerate().fold(
							String::new(),
							|mut s, (i, (name, err))| {
								let _ = write!(s, "\n#{i} {name}: {err:?}"); // can't fail
								s
							}
						)
					)
				})?;
		}
		RunMode::VerifyOnly => {
			tracing::info!("Everything verified to be working properly, exiting...");
		}
		RunMode::MarkOldEntriesAsRead => {
			let mut tasks = tasks;

			// just fetch and save read, don't send anything
			for task in tasks.values_mut() {
				task.inner.sink = None;
			}

			run_tasks(tasks, shutdown_rx, true /* always just run once */, cx)
				.await
				.map_err(|errs| {
					eyre!(
						"{} tasks have finished with an error: {}",
						errs.len(),
						errs.into_iter().enumerate().fold(
							String::new(),
							|mut s, (i, (name, err))| {
								let _ = write!(s, "\n#{i} {name}: {err:?}"); // can't fail
								s
							}
						)
					)
				})?;
		}
	}

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

async fn make_tasks_dry(tasks: &mut ParsedTasks) {
	use fetcher_core::{
		sink::{Sink, Stdout},
		source::{email, Source, WithCustomRF},
	};

	for task in tasks.values_mut() {
		// don't save read filtered items to the fs
		match &mut task.inner.source {
			Source::WithSharedReadFilter { rf, kind: _ } => {
				if let Some(rf) = rf {
					rf.write().await.external_save = None;
				}
			}
			Source::WithCustomReadFilter(custom_rf_source) => match custom_rf_source {
				WithCustomRF::Email(e) => e.view_mode = email::ViewMode::ReadOnly,
			},
		}

		// don't send anything anywhere, just print
		if let Some(sink) = &mut task.inner.sink {
			*sink = Sink::Stdout(Stdout);
		}
	}
}

async fn run_tasks(
	tasks: ParsedTasks,
	shutdown_rx: Receiver<()>,
	once: bool,
	context: Context,
) -> Result<(), Vec<(String, Report)>> {
	let mut running_tasks = Vec::new();
	for (name, mut t) in tasks {
		let name2 = name.clone();
		let mut shutdown_rx = shutdown_rx.clone();

		let task_handle = tokio::spawn(
			async move {
				tracing::trace!("Task {} contents: {:#?}", name, t);

				let res = select! {
					r = task_loop(&mut t, &name, once, context) => r,
					_ = shutdown_rx.changed() => Ok(()),
				};

				// production error reporting
				if !cfg!(debug_assertions) {
					if let Err(err) = &res {
						if let Err(e) = report_error(
							&name,
							&format!("Task {name} stopping with error: {err:#}"),
							context,
						)
						.await
						{
							tracing::error!("Unable to send error report to the admin: {e:?}",);
						}
					}
				}
				tracing::info!("Task {name} shut down...");

				// include the name of the task that has failed in the error
				res.map_err(|res| (name, res))
			}
			.instrument(tracing::info_span!("task", name = name2.as_str())),
		);

		running_tasks.push(flatten_task_result(task_handle));
	}

	let errors = join_all(running_tasks)
		.await
		.into_iter()
		.filter_map(|r| match r {
			Ok(()) => None,
			Err(e) => Some(e),
		})
		.collect::<Vec<_>>();

	if errors.is_empty() {
		Ok(())
	} else {
		Err(errors)
	}
}

async fn task_loop(t: &mut ParsedTask, task_name: &str, once: bool, cx: Context) -> Result<()> {
	// exit with an error if there were too many consecutive errors
	let err_max_count: u32 = if once { 0 } else { 15 }; // around 22 days max pause time

	// number of consecutive(!!!) errors.
	// we tolerate a pretty big amount for various reasons (being rate limited, server error, etc) but not infinite
	let mut err_count = 0;

	loop {
		match fetcher_core::run_task(&mut t.inner).await {
			Ok(()) => {
				err_count = 0;
			}
			Err(err) => {
				if err_count == err_max_count {
					return Err(err.into());
				}

				if let Some(network_err) = err.is_connection_error() {
					tracing::warn!("Network error: {}", network_err.display_chain());
				} else {
					if let Error::Transform(transform_err) = &err {
						settings::log::log_transform_err(transform_err, task_name).await?;
					}

					let err_msg = format!(
						"Error #{} out of {} max allowed:\n{}",
						err_count + 1, // +1 cause we are counting from 0 but it'd be strange to show "Error (0 out of 255)" to users
						err_max_count + 1,
						err.display_chain()
					);
					tracing::error!("{}", err_msg);

					// TODO: make this a context switch
					// production error reporting
					if !cfg!(debug_assertions) {
						if let Err(e) = report_error(task_name, &err_msg, cx).await {
							tracing::error!("Unable to send error report to the admin: {e:?}",);
						}
					}
				}

				// sleep in exponention amount of minutes, begginning with 2^0 = 1 minute
				let sleep_dur = 2u64.saturating_pow(err_count);
				tracing::info!("Pausing task {task_name} for {sleep_dur}m");

				sleep(Duration::from_secs(sleep_dur * 60 /* secs in a min*/)).await;
				err_count += 1;

				continue;
			}
		}

		if once {
			break;
		}

		tracing::debug!(
			"Putting task {task_name} to sleep for {time}m",
			time = t.refresh
		);
		sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
	}

	Ok(())
}

// TODO: move that to a tracing layer that sends all WARN and higher logs automatically
async fn report_error(task_name: &str, err: &str, context: Context) -> Result<()> {
	use fetcher_core::sink::{telegram::LinkLocation, Message, Telegram};

	let admin_chat_id = std::env::var("FETCHER_TELEGRAM_ADMIN_CHAT_ID")
		.wrap_err("FETCHER_TELEGRAM_ADMIN_CHAT_ID")?
		.parse::<i64>()
		.wrap_err("FETCHER_TELEGRAM_ADMIN_CHAT_ID isn't a valid chat id")?;

	let Some(bot) = settings::data::telegram::get(context)? else {
		return Err(eyre!("Telegram bot token not provided"));
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
