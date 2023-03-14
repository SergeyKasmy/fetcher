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
pub mod extentions;
pub mod settings;

use crate::{
	args::{Args, Setting},
	extentions::{ErrorChainExt, SliceDisplayExt},
	settings::{
		config::jobs::filter::JobFilter, context::Context as OwnedContext,
		context::StaticContext as Context,
	},
};
use fetcher_config::jobs::JobName;
use fetcher_core::{
	error::Error,
	job::{timepoint::TimePoint, Job},
	sink::{Sink, Stdout},
};

use color_eyre::{
	eyre::{eyre, WrapErr},
	Report, Result, Section,
};
use futures::future::join_all;
use std::{
	collections::HashMap,
	iter,
	ops::ControlFlow,
	time::{Duration, Instant},
};
use tap::Tap;
use tokio::{
	select,
	sync::watch::{self, Receiver},
	task::JoinHandle,
	time::sleep,
};
use tracing::Instrument;

type Jobs = HashMap<JobName, Job>;

fn main() -> Result<()> {
	set_up_logging()?;
	async_main()
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

	let args: Args = argh::from_env();
	let cx: Context = {
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
		args::TopLvlSubcommand::Run(run_args) => run_command(run_args, cx).await,
		args::TopLvlSubcommand::RunManual(args::RunManual { job }) => {
			run_jobs(
				iter::once(("Manual".to_owned().into(), job.0)),
				ErrorHandling::Forward,
				cx,
			)
			.await?;

			Ok(())
		}
		args::TopLvlSubcommand::MarkOldAsRead(args::MarkOldAsRead { run_filter }) => {
			let run_filter = run_filter
				.into_iter()
				.map(|s| s.parse())
				.collect::<Result<Vec<_>>>()?;
			let run_filter = if run_filter.is_empty() {
				None
			} else {
				Some(run_filter)
			};

			let Some(mut jobs) = get_jobs(run_filter, cx)? else {
				return Ok(());
			};

			// just fetch and save read, don't send anything
			for job in jobs.values_mut() {
				job.refresh_time = None;

				for task in &mut job.tasks {
					task.sink = None;
				}
			}

			run_jobs(jobs, ErrorHandling::LogAndIgnore, cx).await?;
			tracing::info!("Marked jobs as read, exiting...");

			Ok(())
		}
		args::TopLvlSubcommand::Verify(args::Verify { job_run_filter }) => {
			let job_run_filter = job_run_filter
				.into_iter()
				.map(|s| s.parse::<JobFilter>())
				.map(|res| {
					res.map(|mut filter| {
						filter.task = None;
						filter
					})
				})
				.collect::<Result<Vec<_>>>()?;
			let job_run_filter = if job_run_filter.is_empty() {
				None
			} else {
				Some(job_run_filter)
			};

			_ = get_jobs(job_run_filter, cx)?;
			tracing::info!("Everything verified to be working properly, exiting...");

			Ok(())
		}
		args::TopLvlSubcommand::Save(save) => {
			match save.setting {
				Setting::GoogleOAuth2 => settings::data::google_oauth2::prompt(cx).await?,
				Setting::EmailPassword => settings::data::email_password::prompt(cx)?,
				Setting::Telegram => settings::data::telegram::prompt(cx)?,
				Setting::Discord => settings::data::discord::prompt(cx)?,
				Setting::Twitter => settings::data::twitter::prompt(cx)?,
			}

			Ok(())
		}
	}
}

async fn run_command(
	args::Run {
		once,
		dry_run,
		run_filter,
	}: args::Run,
	cx: Context,
) -> Result<()> {
	let Some(mut jobs) = ({
		let run_filter = run_filter.into_iter().map(|s| s.parse()).collect::<Result<Vec<_>>>()?;
		let run_filter = if run_filter.is_empty() {
			None
		} else {
			Some(run_filter)
		};

		get_jobs(run_filter, cx)?
	}) else {
		return Ok(());
	};

	if dry_run {
		tracing::trace!("Making all jobs dry");

		for job in jobs.values_mut() {
			for task in &mut job.tasks {
				// don't save read filtered items to the fs
				if let Some(source) = &mut task.source {
					source.set_read_only().await;
				}

				// don't send anything anywhere, just print
				if let Some(sink) = &mut task.sink {
					*sink = Box::new(Stdout);
				}

				// don't save entry to msg map to the fs
				if let Some(entry_to_msg_map) = &mut task.entry_to_msg_map {
					entry_to_msg_map.external_save = None;
				}
			}
		}
	}

	if once {
		tracing::trace!("Disabling every job's refetch interval");

		for job in jobs.values_mut() {
			job.refresh_time = None;
		}
	}

	let error_handling = if once {
		ErrorHandling::Forward
	} else {
		ErrorHandling::Sleep {
			max_retries: 15,
			err_count: 0,
			last_error: None,
		}
	};

	run_jobs(jobs, error_handling, cx).await?;
	Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn get_jobs(run_filter: Option<Vec<JobFilter>>, cx: Context) -> Result<Option<Jobs>> {
	let run_by_name_is_some = run_filter.is_some();
	let jobs = settings::config::jobs::get_all(run_filter.as_deref(), cx)?;

	if run_by_name_is_some {
		if jobs.is_empty() {
			tracing::info!("No enabled jobs found for the provided query");

			let all_jobs = settings::config::jobs::get_all(None, cx)?;
			tracing::info!("All available enabled jobs: {}", all_jobs.keys().display());

			return Ok(None);
		}

		tracing::info!(
			"Found {} enabled jobs for the provided query: {}",
			jobs.len(),
			jobs.keys().display()
		);
	} else {
		if jobs.is_empty() {
			tracing::info!("No enabled jobs found");
			return Ok(None);
		}

		tracing::info!(
			"Found {} enabled jobs: {}",
			jobs.len(),
			jobs.keys().display()
		);
	}

	tracing::trace!("Jobs to run: {jobs:?}");
	Ok(Some(jobs))
}

#[derive(Clone, Debug)]
enum ErrorHandling {
	Forward,
	LogAndIgnore,
	Sleep {
		max_retries: u32,

		// "private" state, should be 0 and None
		// there's no point in creating a private struct with a constructor just for these
		// since they are for private use anyways and aren't used more than a couple of times
		err_count: u32,
		last_error: Option<Instant>,
	},
}

async fn run_jobs(
	jobs: impl IntoIterator<Item = (JobName, Job)>,
	error_handling: ErrorHandling,
	cx: Context,
) -> Result<()> {
	let shutdown_rx = set_up_signal_handler();

	let jobs = jobs
		.into_iter()
		.map(|(name, mut job)| {
			let async_job = {
				let name = name.clone();
				let mut error_handling = error_handling.clone();

				async move {
					loop {
						let job_result = job
							.run()
							.instrument(tracing::info_span!("job", name = %name))
							.await;

						match handle_errors(job_result, &mut error_handling, (&name, &job), cx)
							.await
						{
							ControlFlow::Continue(()) => (),
							ControlFlow::Break(res) => return res.map_err(|e| (name, e)),
						}
					}
				}
			};

			// tokio task
			let async_task = {
				let mut shutdown_rx = shutdown_rx.clone();

				async move {
					loop {
						select! {
							res = async_job => {
								return res;
							}
							_ = shutdown_rx.changed() => {
								tracing::info!("Job {name} shut down...");
								return Ok(());
							}
						}
					}
				}
			};

			let async_task_handle = tokio::spawn(async_task);
			flatten_task_result(async_task_handle)
		})
		.collect::<Vec<_>>();

	// rust-analyzer is confused without these manual type annotation
	let mut errors: Vec<(JobName, Report)> = join_all(jobs)
		.await
		.into_iter()
		.filter_map(|r| match r {
			Ok(()) => None,
			Err(e) => Some(e),
		})
		.collect();

	match errors.len() {
		0 => Ok(()),
		1 => {
			let (name, error) = errors.pop().expect("len should be 1");

			Err(error).wrap_err(format!("Job \"{name}\""))
		}
		i => {
			let full_report = errors.into_iter().fold(
				eyre!("{i} jobs have finished with an error"),
				|acc, (name, err)| acc.report(err.wrap_err(format!("Job \"{name}\""))),
			);

			Err(full_report)
		}
	}
}

fn set_up_signal_handler() -> Receiver<()> {
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
			.expect("failed to broadcast shutdown signal to the jobs");

		tracing::info!("Press Ctrl-C again to force close");

		// force close
		tokio::signal::ctrl_c()
			.await
			.expect("failed to setup signal handler");

		force_close_tx
			.send(())
			.expect("failed to broadcast force shutdown signal");

		Ok::<_, Report>(())
	});

	// force close signal receiver
	tokio::spawn(async move {
		if let Ok(()) = force_close_rx.changed().await {
			tracing::info!("Force closing...");
			std::process::exit(1);
		}
	});

	shutdown_rx
}

async fn handle_errors(
	results: Result<(), Vec<Error>>,
	stradegy: &mut ErrorHandling,
	(job_name, job): (&JobName, &Job),
	cx: Context,
) -> ControlFlow<Result<()>> {
	let Err(errors) = results else {
		return ControlFlow::Break(Ok(()));
	};

	match stradegy {
		ErrorHandling::Forward => (),
		ErrorHandling::LogAndIgnore => {
			for error in &errors {
				tracing::error!("{error:?}");
			}

			return ControlFlow::Continue(());
		}
		ErrorHandling::Sleep {
			max_retries,
			err_count,
			last_error,
		} => {
			if let Some(last_error_instant) = last_error {
				if let Some(refresh_time) = &job.refresh_time {
					// if time since last error is 2 times longer than the refresh duration, than the error count can safely be reset
					// since there hasn't been any errors for a little while
					// TODO: maybe figure out a more optimal time interval than just 2 times longer than the refresh timer

					match refresh_time {
						TimePoint::Duration(dur) => {
							if last_error_instant.elapsed() > (*dur * 2) {
								*err_count = 0;
								*last_error = None;
							}
						}
						// once a day
						TimePoint::Time(_) => {
							const TWO_DAYS: Duration = Duration::from_secs(
								2 /* days */ * 24 /* hours a day */ * 60 /* mins an hour */ * 60, /* secs a min */
							);

							if last_error_instant.elapsed() > TWO_DAYS {
								*err_count = 0;
								*last_error = None;
							}
						}
					}
				}
			}

			for err in errors {
				if err_count == max_retries {
					return ControlFlow::Break(Err(err.into()));
				}

				if let Some(network_err) = err.is_connection_error() {
					tracing::warn!("Network error: {}", network_err.display_chain());
				} else {
					if let Error::Transform(transform_err) = &err {
						if let Err(e) = settings::log::log_transform_err(transform_err, job_name) {
							tracing::error!("Error logging transform error: {e:?}");
						}
					}

					let err_msg = format!(
						"Error #{} out of {} max allowed:\n{}",
						*err_count + 1, // +1 cause we are counting from 0 but it'd be strange to show "Error (0 out of 255)" to users
						*max_retries,
						err.display_chain()
					);
					tracing::error!("{}", err_msg);

					// TODO: make this a context switch
					// production error reporting
					if !cfg!(debug_assertions) {
						if let Err(e) = report_error(job_name, &err_msg, cx).await {
							tracing::error!("Unable to send error report to the admin: {e:?}",);
						}
					}
				}

				// sleep in exponention amount of minutes, begginning with 2^0 = 1 minute
				let sleep_dur = 2u64.saturating_pow(*err_count);
				tracing::info!("Pausing job {job_name} for {sleep_dur}m");

				*err_count += 1;
				sleep(Duration::from_secs(sleep_dur * 60 /* secs in a min */)).await;
			}

			return ControlFlow::Continue(());
		}
	}

	// no point in making errors mutable for the duration of the whole functions if it's needed just down here
	let mut errors = errors;

	// for acc_report.error(err). I believe this way it is clearer what the fold does
	#[allow(clippy::redundant_closure_for_method_calls)]
	let full_report = match errors.len() {
		0 => unreachable!(),
		1 => Report::from(errors.remove(0)),
		i => errors.into_iter().fold(
			eyre!("{i} tasks have finished with an error"),
			|acc_report, err| acc_report.error(err),
		),
	};

	ControlFlow::Break(Err(full_report))
}

// TODO: move that to a tracing layer that sends all WARN and higher logs automatically
async fn report_error(job_name: &str, err: &str, context: Context) -> Result<()> {
	use fetcher_core::sink::{message::Message, telegram::LinkLocation, Telegram};

	let admin_chat_id = std::env::var("FETCHER_TELEGRAM_ADMIN_CHAT_ID")
		.wrap_err("FETCHER_TELEGRAM_ADMIN_CHAT_ID")?
		.parse::<i64>()
		.wrap_err("FETCHER_TELEGRAM_ADMIN_CHAT_ID isn't a valid chat id")?;

	let Ok(bot) = settings::data::telegram::get(context) else {
		return Err(eyre!("Telegram bot token not provided"));
	};

	let msg = Message {
		body: Some(err.to_owned()),
		..Default::default()
	};
	Telegram::new(bot, admin_chat_id, LinkLocation::default())
		.send(msg, None, Some(job_name))
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
