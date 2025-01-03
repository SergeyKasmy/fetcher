/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![doc = include_str!("../README.md")]
#![feature(let_chains)]
#![allow(missing_docs)] // TODO: add more docs
#![allow(clippy::missing_docs_in_private_items)] // TODO: enable later
#![allow(clippy::missing_errors_doc)] // TODO: add more docs
#![allow(clippy::missing_panics_doc)] // TODO: add more docs
#![allow(clippy::future_not_send)] // not useful in a binary crate

pub mod args;
pub mod error_handling;
pub mod extentions;
pub mod settings;

use crate::{
	args::{Args, Setting},
	error_handling::{DEFAULT_MAX_ERROR_LIMIT, ErrorHandling, PrevErrors},
	extentions::{ErrorChainExt, SliceDisplayExt, slice_display::job_display::JobDisplay},
	settings::{
		config::jobs::filter::JobFilter, context::Context as OwnedContext,
		context::StaticContext as Context,
	},
};
use fetcher_config::jobs::named::{JobName, JobWithTaskNames};
use fetcher_core::{
	action::Action,
	error::FetcherError,
	job::{Job, timepoint::TimePoint},
	sink::{Sink, Stdout},
};

use color_eyre::{
	Report, Result, Section,
	eyre::{WrapErr, eyre},
};
use futures::{StreamExt, stream::FuturesUnordered};
use std::{collections::HashMap, fmt::Write, ops::ControlFlow, path::PathBuf, time::Duration};
use tap::TapOptional;
use tokio::{
	select,
	sync::watch::{self, Receiver},
	task::JoinError,
	time::sleep,
};
use tracing::Instrument;

type Jobs = HashMap<JobName, JobWithTaskNames>;

fn main() -> Result<()> {
	set_up_logging()?;
	async_main()
}

fn set_up_logging() -> Result<()> {
	use tracing::Level;
	use tracing_subscriber::{
		EnvFilter, Layer, filter::LevelFilter, fmt::time::OffsetTime, layer::SubscriberExt,
	};

	let env_filter =
		EnvFilter::try_from_env("FETCHER_LOG").unwrap_or_else(|_| EnvFilter::from("info"));

	let is_debug_log_level = env_filter
		.max_level_hint()
		.map_or_else(|| false, |level| level >= Level::DEBUG);

	let stdout = tracing_subscriber::fmt::layer()
		.with_target(is_debug_log_level)
		.with_file(is_debug_log_level)
		.with_line_number(is_debug_log_level)
		.with_thread_ids(is_debug_log_level)
		.with_timer(OffsetTime::local_rfc_3339().expect("could not get local time offset"));

	let stdout = if is_debug_log_level {
		stdout.pretty().boxed()
	} else {
		stdout.boxed()
	};

	// enable journald logging only on release to avoid log spam on dev machines
	let journald = if cfg!(debug_assertions) {
		None
	} else {
		tracing_journald::layer().ok()
	};

	let subscriber = tracing_subscriber::registry()
		.with(journald.with_filter(LevelFilter::INFO))
		.with(stdout.with_filter(env_filter));

	tracing::subscriber::set_global_default(subscriber)
		.expect("tracing shouldn't already have been set up");

	color_eyre::install()?;
	Ok(())
}

#[tokio::main]
async fn async_main() -> Result<()> {
	// TODO: move these to the actual main and just make async_main take (args: Args, version: String)?
	let args: Args = argh::from_env();
	let version = version();

	if args.print_version {
		println!("fetcher {version}");
		return Ok(());
	}

	let cx = create_context(args.data_path, args.config_path, args.log_path)?;
	tracing::info!("Running fetcher {version}");

	match args.subcommand {
		Some(args::TopLvlSubcommand::Run(run_args)) => run_command(run_args, cx).await,
		None => run_command(args::Run::default(), cx).await,
		Some(args::TopLvlSubcommand::RunManual(args::RunManual { job_config })) => {
			run_jobs(job_config.decode(cx)?, ErrorHandling::Forward, cx).await?;

			Ok(())
		}
		Some(args::TopLvlSubcommand::MarkOldAsRead(args::MarkOldAsRead { run_filter })) => {
			let run_filter = run_filter
				.into_iter()
				.map(|s| s.parse())
				.collect::<Result<Vec<JobFilter>>>()?;
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
				job.inner.refresh_time = None;

				for task in &mut job.inner.tasks {
					if let Some(actions) = task.actions.take() {
						let no_sink_acts = actions
							.into_iter()
							.filter(|a| !matches!(a, Action::Sink(_)))
							.collect::<Vec<_>>();

						if !no_sink_acts.is_empty() {
							task.actions = Some(no_sink_acts);
						}
					}
				}
			}

			run_jobs(jobs, ErrorHandling::LogAndIgnore, cx).await?;
			tracing::info!("Marked jobs as read, exiting...");

			Ok(())
		}
		Some(args::TopLvlSubcommand::Verify(args::Verify { job_run_filter })) => {
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

			let _: Option<Jobs> = get_jobs(job_run_filter, cx)?;
			tracing::info!("Everything verified to be working properly, exiting...");

			Ok(())
		}
		Some(args::TopLvlSubcommand::Save(save)) => {
			match save.setting {
				Setting::GoogleOAuth2 => settings::data::google_oauth2::prompt(cx).await?,
				Setting::EmailPassword => settings::data::email_password::prompt(cx)?,
				Setting::Telegram => settings::data::telegram::prompt(cx)?,
				Setting::Discord => settings::data::discord::prompt(cx)?,
			}

			Ok(())
		}
	}
}

/// Override default path with a custom one if it is Some
fn create_context(
	data_path: Option<PathBuf>,
	config_path: Option<PathBuf>,
	log_path: Option<PathBuf>,
) -> Result<Context> {
	let data_path = match data_path {
		Some(p) => p,
		None => settings::data::default_data_path()?,
	};
	let conf_paths = match config_path {
		Some(p) => vec![p],
		None => settings::config::default_cfg_dirs()?,
	};
	let log_path = match log_path {
		Some(p) => p,
		None => settings::log::default_log_path()?,
	};

	Ok(Box::leak(Box::new(OwnedContext {
		data_path,
		conf_paths,
		log_path,
	})))
}

fn version() -> String {
	// no, clippy, just using env!() won't work here since we are running it conditionally and it doesn't always exist in all branches
	#[allow(clippy::option_env_unwrap)]
	match (
		option_env!("FETCHER_MAIN_BRANCH_OVERRIDE").is_some(),
		option_env!("VERGEN_GIT_BRANCH"),
	) {
		// if main branch override isn't set and the branch isn't main
		(false, Some(branch)) if branch != "main" => format!(
			"v{}-{} on branch {branch}",
			env!("CARGO_PKG_VERSION"),
			option_env!("VERGEN_GIT_SHA")
				.expect("vergen should've run successfully if VERGEN_GIT_BRANCH is set"),
		),
		_ => concat!("v", env!("CARGO_PKG_VERSION")).to_owned(),
	}
}

async fn run_command(run_args: args::Run, cx: Context) -> Result<()> {
	tracing::trace!("Running in run mode with {run_args:#?}");

	let args::Run {
		once,
		no_skip_read: ignore_read,
		dry_run,
		run_filter,
	} = run_args;

	let run_filter = {
		let run_filter = run_filter
			.into_iter()
			.map(|s| s.parse())
			.collect::<Result<Vec<JobFilter>>>()?;

		if run_filter.is_empty() {
			None
		} else {
			Some(run_filter)
		}
	};

	let Some(mut jobs) = get_jobs(run_filter, cx)? else {
		return Ok(());
	};

	if once {
		tracing::trace!("Disabling every job's refresh time");

		for job in jobs.values_mut() {
			job.inner.refresh_time = None;
		}
	}

	if ignore_read {
		tracing::trace!("Disabling read filters");
		for job in jobs.values_mut() {
			for task in &mut job.inner.tasks {
				let Some(actions) = task.actions.take() else {
					continue;
				};

				// TODO: use .retain mb
				let new_actions = actions
					.into_iter()
					.filter(|act| {
						if let Action::Filter(filter) = &act
							&& filter.is_readfilter()
						{
							return false;
						}

						true
					})
					.collect::<Vec<_>>();

				if !new_actions.is_empty() {
					task.actions = Some(new_actions);
				}
			}
		}
	}

	if dry_run {
		tracing::trace!("Making all jobs dry");

		for job in jobs.values_mut() {
			for task in &mut job.inner.tasks {
				// don't save read filtered items to the fs
				if let Some(source) = &mut task.source {
					source.set_read_only().await;
				}

				// don't send anything anywhere, just print
				for act in task.actions.iter_mut().flatten() {
					if let Action::Sink(sink) = act {
						*sink = Box::new(Stdout);
					}
				}

				// don't save entry to msg map to the fs
				if let Some(entry_to_msg_map) = &mut task.entry_to_msg_map {
					entry_to_msg_map.external_save = None;
				}
			}
		}
	}

	let error_handling = if once {
		ErrorHandling::Forward
	} else {
		ErrorHandling::Sleep {
			prev_errors: PrevErrors::new(DEFAULT_MAX_ERROR_LIMIT),
		}
	};

	run_jobs(jobs, error_handling, cx).await?;
	Ok(())
}

#[tracing::instrument(level = "debug", skip(cx))]
#[allow(clippy::needless_pass_by_value)]
fn get_jobs(run_filter: Option<Vec<JobFilter>>, cx: Context) -> Result<Option<Jobs>> {
	let run_by_name_is_some = run_filter.is_some();
	let jobs = settings::config::jobs::get_all(run_filter.as_deref(), cx)?;

	if run_by_name_is_some {
		if jobs.is_empty() {
			tracing::info!("No enabled jobs found for the provided query");

			if let Ok(all_jobs) = settings::config::jobs::get_all(None, cx) {
				// tracing::info!("All available enabled jobs: {}", all_jobs.keys().display());
				tracing::info!(
					"All available enabled jobs: {}",
					all_jobs.iter().map(JobDisplay).display()
				);
			} else {
				tracing::warn!(
					"Can't list all available jobs because some jobs have invalid format. Try running in \"verify\" mode and correcting them"
				);
			}

			return Ok(None);
		}

		tracing::info!(
			"Found {} enabled jobs for the provided query: {}",
			jobs.len(),
			jobs.iter().map(JobDisplay).display()
		);
	} else {
		if jobs.is_empty() {
			tracing::info!("No enabled jobs found");
			return Ok(None);
		}

		tracing::info!(
			"Found {} enabled jobs: {}",
			jobs.len(),
			jobs.iter().map(JobDisplay).display()
		);
	}

	tracing::trace!("Jobs to run: {jobs:#?}");
	Ok(Some(jobs))
}

#[tracing::instrument(level = "trace", skip_all)]
async fn run_jobs(
	jobs: impl IntoIterator<Item = (JobName, JobWithTaskNames)>,
	error_handling: ErrorHandling,
	cx: Context,
) -> Result<()> {
	let shutdown_rx = set_up_signal_handler();

	let jobs = jobs
		.into_iter()
		.map(|(name, job)| {
			run_job(
				name,
				job.inner,
				error_handling.clone(),
				shutdown_rx.clone(),
				cx,
			)
		})
		.collect::<FuturesUnordered<_>>();

	let mut errors: Vec<(JobName, Report)> = jobs
		.filter_map(|(job_name, async_task_res)| async move {
			if let Ok(job_res) = async_task_res {
				match job_res {
					Ok(()) => {
						tracing::info!("Job {job_name} has finished");
						None
					}
					Err(e) => {
						tracing::error!("Job {job_name} has exited with an error: {e:?}");
						Some((job_name, e))
					}
				}
			} else {
				tracing::error!("Job {job_name} has crashed");
				None
			}
		})
		.collect()
		.await;

	match errors.len() {
		0 => Ok(()),
		1 => {
			let (name, error) = errors.pop().expect("len should be 1");

			Err(error).wrap_err(format!("Job {name}"))
		}
		i => {
			let full_report = errors.into_iter().fold(
				eyre!("{i} jobs have exited with an error"),
				|acc, (name, err)| acc.report(err.wrap_err(format!("Job {name}"))),
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
			#[allow(clippy::exit)] // user requested force-close, it's allowed here
			std::process::exit(1);
		}
	});

	shutdown_rx
}

async fn run_job(
	name: JobName,
	mut job: Job,
	mut error_handling: ErrorHandling,
	mut shutdown_rx: Receiver<()>,
	cx: Context,
) -> (JobName, Result<Result<()>, JoinError>) {
	fn fold_task_errors(mut errors: Vec<FetcherError>) -> Report {
		// for acc_report.error(err). I believe this way it is clearer what the fold does
		#[allow(clippy::redundant_closure_for_method_calls)]
		match errors.len() {
			0 => panic!("Empty error vec which is a programmer error, this should never happen"),
			1 => Report::from(errors.remove(0)),
			i => errors.into_iter().fold(
				eyre!("{i} tasks have exited with an error"),
				|acc_report, err| acc_report.error(err),
			),
		}
	}

	let async_job = {
		let name = name.clone();

		async move {
			loop {
				let job_result = job.run().await;

				match handle_errors(job_result, &mut error_handling, (&name, &job), cx).await {
					ControlFlow::Continue(()) => (),
					ControlFlow::Break(res) => {
						return res.map_err(fold_task_errors);
					}
				}
			}
		}
	};

	// tokio task
	let async_task = {
		let name = name.clone();

		async move {
			#[allow(clippy::redundant_pub_crate)] // false positive
			{
				select! {
					res = async_job => {
						res
					}
					_ = shutdown_rx.changed() => {
						tracing::info!("Job {name} signaled to shutdown...");
						Ok(())
					}
				}
			}
		}
	}
	.instrument(tracing::info_span!("job", name = %name));

	(name, tokio::spawn(async_task).await)
}

/// ControlFlow::Continue -> continue running the job
/// ControlFlow::Break -> stop running the job with a result
#[tracing::instrument(level = "debug", skip(job_name, job, cx))]
async fn handle_errors(
	results: Result<(), Vec<FetcherError>>,
	stradegy: &mut ErrorHandling,
	(job_name, job): (&JobName, &Job),
	cx: Context,
) -> ControlFlow<Result<(), Vec<FetcherError>>> {
	let Err(errors) = results else {
		return ControlFlow::Break(Ok(()));
	};

	match stradegy {
		ErrorHandling::Forward => {
			tracing::trace!("Forwarding errors");

			ControlFlow::Break(Err(errors))
		}
		ErrorHandling::LogAndIgnore => {
			for error in &errors {
				tracing::error!("{}", error.display_chain());
			}

			ControlFlow::Continue(())
		}
		ErrorHandling::Sleep { prev_errors } => {
			match handle_errors_sleep(
				&errors,
				prev_errors,
				job_name,
				job.refresh_time.as_ref(),
				cx,
			)
			.await
			{
				ControlFlow::Continue(()) => ControlFlow::Continue(()),
				ControlFlow::Break(()) => ControlFlow::Break(Err(errors)),
			}
		}
	}
}

// count errors and sleep exponentially
async fn handle_errors_sleep(
	errors: &[FetcherError],
	prev_errors: &mut PrevErrors,
	job_name: &JobName,
	job_refresh_time: Option<&TimePoint>,
	cx: Context,
) -> ControlFlow<()> {
	// if time since last error is 2 times longer than the refresh duration, then the error count can safely be reset
	// since there hasn't been any errors for a little while
	// TODO: maybe figure out a more optimal time interval than just 2 times longer than the refresh timer
	if let Some((last_error, refresh_time)) = prev_errors.last_error().zip(job_refresh_time) {
		let last_error_sleep_dur = exponential_backoff_duration(prev_errors.count());
		match refresh_time {
			TimePoint::Duration(dur) => {
				let twice_refresh_dur = *dur * 2; // two times the refresh duration to make sure the job ran at least twice with no errors
				if last_error.elapsed() > last_error_sleep_dur + twice_refresh_dur {
					prev_errors.reset();
				}
			}
			// once a day
			TimePoint::Time(_) => {
				const TWO_DAYS: Duration = Duration::from_secs(
					2 /* days */ * 24 /* hours a day */ * 60 /* mins an hour */ * 60, /* secs a min */
				);

				if last_error.elapsed() > last_error_sleep_dur + TWO_DAYS {
					prev_errors.reset();
				}
			}
		}
	}

	// log and filter out network connection errors.
	// they shouldn't be counted against the max error limit because they are ~usually~ temporary and not critical
	let errors_without_net = errors.iter().filter(|e| {
		e.is_connection_error()
			.tap_some(|net_err| {
				tracing::warn!("Network error: {}", net_err.display_chain());
			})
			.is_none()
	});

	if errors_without_net.clone().count() > 0 {
		// max error limit reached
		if prev_errors.push() {
			tracing::warn!(
				"Maximum error limit reached ({max} out of {max}) for job {job_name}. Stopping retrying...",
				max = prev_errors.max_retries
			);
			return ControlFlow::Break(());
		}

		let mut err_msg = format!(
			"Job {job_name} finished {job_err_count} times in an error (out of {max} max allowed)",
			job_err_count = prev_errors.count(),
			max = prev_errors.max_retries,
		);

		// log and report all other errors (except for network errors up above)
		for (i, err) in errors_without_net.enumerate() {
			if let FetcherError::Transform(transform_err) = &err
				&& let Err(e) = settings::log::log_transform_err(transform_err, job_name)
			{
				tracing::error!("Error logging transform error: {e:?}");
			}

			_ = write!(
				err_msg,
				"\nError #{err_num}:\n{e}\n",
				err_num = i + 1,
				e = err.display_chain()
			);
		}

		tracing::error!("{}", err_msg);

		// TODO: make this a context switch
		// production error reporting
		if !cfg!(debug_assertions) {
			if let Err(e) = report_error(job_name, &err_msg, cx).await {
				tracing::error!("Unable to send error report to the admin: {e:?}",);
			}
		}
	}

	let sleep_dur = exponential_backoff_duration(prev_errors.count());
	tracing::info!("Pausing job {job_name} for {}m", sleep_dur.as_secs() / 60);
	sleep(sleep_dur).await;

	ControlFlow::Continue(())
}

/// Sleep in exponentially increasing amount of minutes, beginning with 2^0 = 1 minute.
const fn exponential_backoff_duration(consecutive_err_count: u32) -> Duration {
	// subtract 1 because prev_errors.count() is already set to 1 (because the first error has already happened)
	// but we want to sleep beginning with ^0, not ^1
	let sleep_dur = 2u64.saturating_pow(consecutive_err_count.saturating_sub(1));
	Duration::from_secs(sleep_dur * 60 /* secs in a min */)
}

// TODO: move that to a tracing layer that sends all WARN and higher logs automatically
async fn report_error(job_name: &str, err: &str, context: Context) -> Result<()> {
	use fetcher_core::sink::{Telegram, message::Message, telegram::LinkLocation};

	let admin_chat_id = std::env::var("FETCHER_TELEGRAM_ADMIN_CHAT_ID")
		.wrap_err("FETCHER_TELEGRAM_ADMIN_CHAT_ID not set")?
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
		.send(&msg, None, Some(job_name))
		.await
		.map_err(fetcher_core::error::FetcherError::Sink)?;

	Ok(())
}
