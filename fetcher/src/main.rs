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

use self::settings::{context::Context as OwnedContext, context::StaticContext as Context};
use crate::args::{Args, Setting};
use fetcher_core::{
	error::{Error, ErrorChainExt},
	job::Job,
	sink::{Sink, Stdout},
	source::{email, Source, WithCustomRF},
};

use color_eyre::{
	eyre::{eyre, WrapErr},
	Report, Result,
};
use futures::future::join_all;
use std::{
	collections::HashMap,
	fmt::Write,
	iter,
	time::{Duration, Instant},
};
use tokio::{
	select,
	sync::watch::{self, Receiver},
	task::JoinHandle,
	time::sleep,
};

pub type JobName = String;
pub type Jobs = HashMap<JobName, Job>;

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
				iter::once(("Manual".to_owned(), job.0)),
				ErrorHandling::Forward,
				cx,
			)
			.await?;

			Ok(())
		}
		args::TopLvlSubcommand::MarkOldAsRead(args::MarkOldAsRead {}) => {
			// TODO: add verify by name
			let Some(mut jobs) = get_jobs(None, cx)? else {
				return Ok(());
			};

			// just fetch and save read, don't send anything
			for job in jobs.values_mut() {
				job.refetch_interval = None;

				for task in &mut job.tasks {
					task.sink = None;
				}
			}

			run_jobs(jobs, ErrorHandling::LogAndIgnore, cx).await?;

			Ok(())
		}
		args::TopLvlSubcommand::Verify(args::Verify {}) => {
			// TODO: add verify by name
			let _ = get_jobs(None, cx)?;
			tracing::info!("Everything verified to be working properly, exiting...");

			Ok(())
		}
		args::TopLvlSubcommand::Save(save) => {
			match save.setting {
				Setting::GoogleOAuth2 => settings::data::google_oauth2::prompt(cx).await?,
				Setting::EmailPassword => settings::data::email_password::prompt(cx)?,
				Setting::Telegram => settings::data::telegram::prompt(cx)?,
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
		job_names,
	}: args::Run,
	cx: Context,
) -> Result<()> {
	let Some(mut jobs) = ({
		let run_by_name = if job_names.is_empty() {
			None
		} else {
			Some(job_names)
		};

		get_jobs(run_by_name, cx)?
	}) else {
		return Ok(());
	};

	if dry_run {
		tracing::debug!("Making all jobs dry");

		for job in jobs.values_mut() {
			for task in &mut job.tasks {
				// don't save read filtered items to the fs
				match &mut task.source {
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
				if let Some(sink) = &mut task.sink {
					*sink = Sink::Stdout(Stdout);
				}
			}
		}
	}

	if once {
		tracing::debug!("Disabling every job's refetch interval");

		for job in jobs.values_mut() {
			job.refetch_interval = None;
		}
	}

	run_jobs(
		jobs,
		ErrorHandling::Sleep {
			max_retries: 15,
			err_count: 0,
			last_error: None,
		},
		cx,
	)
	.await?;
	Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn get_jobs(run_by_name: Option<Vec<String>>, cx: Context) -> Result<Option<Jobs>> {
	let run_by_name_is_some = run_by_name.is_some();
	// FIXME ::tasks:: -> ::jobs::
	let jobs = settings::config::tasks::get_all(
		run_by_name
			.as_ref()
			.map(|s| s.iter().map(String::as_str).collect::<Vec<_>>())
			.as_deref(),
		cx,
	)?;

	if run_by_name_is_some {
		if jobs.is_empty() {
			tracing::info!("No enabled jobs found for the provided query");

			let all_jobs = settings::config::tasks::get_all(
				run_by_name
					.as_ref()
					.map(|s| s.iter().map(String::as_str).collect::<Vec<_>>())
					.as_deref(),
				cx,
			)?;
			tracing::info!(
				"All available enabled jobs: {:?}",
				all_jobs.keys().collect::<Vec<_>>()
			);

			return Ok(None);
		}

		tracing::info!(
			"Found {} enabled jobs for the provided query: {:?}",
			jobs.len(),
			jobs.keys().collect::<Vec<_>>()
		);
	} else {
		if jobs.is_empty() {
			tracing::info!("No enabled jobs found");
			return Ok(None);
		}

		tracing::info!(
			"Found {} enabled jobs: {:?}",
			jobs.len(),
			jobs.keys().collect::<Vec<_>>()
		);
	}

	Ok(Some(jobs))
}

#[derive(Clone, Debug)]
enum ErrorHandling {
	Forward,
	LogAndIgnore,
	Sleep {
		max_retries: u32,

		// "private" state, should be 0 and None
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
			let mut shutdown_rx = shutdown_rx.clone();
			let mut error_handling = error_handling.clone();

			let async_task = async move {
				loop {
					select! {
						res = job.run() => {
							handle_errors(&mut error_handling, res, &job, cx).await.map_err(|e| (name.clone(), e))?;
						}
						_ = shutdown_rx.changed() => {
							tracing::info!("Job {name} shut down...");
							return Ok(());
						}
					}
				}
			};

			let async_task_handle = tokio::spawn(async_task);
			flatten_task_result(async_task_handle)
		})
		.collect::<Vec<_>>();

	// rust-analyzer is confused without these manual type annotation
	let errors: Vec<(JobName, Report)> = join_all(jobs)
		.await
		.into_iter()
		.filter_map(|r| match r {
			Ok(()) => None,
			Err(e) => Some(e),
		})
		.collect();

	match errors.as_slice() {
		[] => Ok(()),
		[(name, error)] => Err(eyre!("Job \"{name}\" has finished with an error: {error}")),
		errors => {
			Err(eyre!(
				"{} jobs have finished with an error: {}",
				errors.len(),
				errors
					.iter()
					.enumerate()
					.fold(String::new(), |mut err_str, (i, (name, error))| {
						let _ = write!(err_str, "\n#{} {name}: {error}", i + 1); // can't fail
						err_str
					})
			))
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
		force_close_rx
			.changed()
			.await
			.expect("force close transmitter has been closed");

		tracing::info!("Force closing...");

		std::process::exit(1);
	});

	shutdown_rx
}

async fn handle_errors(
	stradegy: &mut ErrorHandling,
	results: Result<(), Vec<Error>>,
	job: &Job,
	cx: Context,
) -> Result<()> {
	let Err(errors) = results else {
		return Ok(());
	};

	match stradegy {
		ErrorHandling::Forward => (),
		ErrorHandling::LogAndIgnore => {
			for error in &errors {
				tracing::error!("{error:?}");
			}

			return Ok(());
		}
		ErrorHandling::Sleep {
			max_retries,
			err_count,
			last_error,
		} => {
			if let Some(last_error) = last_error {
				if let Some(refetch_interval) = job.refetch_interval {
					// if time since last error is 2 times longer than the refresh duration, than the error count can safely be reset
					// since there hasn't been any errors for a little while
					// TODO: maybe figure out a more optimal time interval than just 2 times longer than the refresh timer
					if last_error.elapsed() > refetch_interval * 2 {
						*err_count = 0;
					}
				}
			}

			for err in errors {
				if err_count == max_retries {
					return Err(eyre!(err.display_chain()));
				}

				if let Some(network_err) = err.is_connection_error() {
					tracing::warn!("Network error: {}", network_err.display_chain());
				} else {
					if let Error::Transform(transform_err) = &err {
						// settings::log::log_transform_err(transform_err, &job_name)?;
						settings::log::log_transform_err(transform_err, "FIXME")?;
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
						// if let Err(e) = report_error(&job_name, &err_msg, cx).await {
						if let Err(e) = report_error("FIXME", &err_msg, cx).await {
							tracing::error!("Unable to send error report to the admin: {e:?}",);
						}
					}
				}

				// sleep in exponention amount of minutes, begginning with 2^0 = 1 minute
				let sleep_dur = 2u64.saturating_pow(*err_count);
				// tracing::info!("Pausing job {job_name} for {sleep_dur}m");
				tracing::info!("Pausing job FIXME for {sleep_dur}m");

				*err_count += 1;
				sleep(Duration::from_secs(sleep_dur * 60 /* secs in a min */)).await;
			}

			return Ok(());
		}
	}

	Err(fold_task_errors(&errors))
}

fn fold_task_errors(task_errors: &[Error]) -> Report {
	match task_errors {
		[] => unreachable!(),
		[error] => {
			// FIXME: use .into()
			eyre!(error.display_chain())
		}
		errors => {
			let combined_err_str =
				errors
					.iter()
					.enumerate()
					.fold(String::new(), |mut err_str, (i, err)| {
						let _ = write!(err_str, "\n{}: {}", i + 1, err.display_chain());
						err_str
					});

			eyre!(
				"{} tasks have finished with an error: {combined_err_str}",
				errors.len()
			)
		}
	}
}

// TODO: move that to a tracing layer that sends all WARN and higher logs automatically
async fn report_error(job_name: &str, err: &str, context: Context) -> Result<()> {
	use fetcher_core::sink::{telegram::LinkLocation, Message, Telegram};

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
		.send(msg, Some(job_name))
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
