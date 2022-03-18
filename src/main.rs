/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

mod settings;

use fetcher::{
	config::{self, DataSettings},
	error::{Error, Result},
	read_filter::ReadFilter,
	run_task,
	task::{NamedTask, Tasks},
};
use futures::future::join_all;
use futures::StreamExt;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook_tokio::Signals;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::{select, sync::watch::Receiver};
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

use crate::settings::data::{
	generate_google_oauth2, generate_google_password, generate_telegram, generate_twitter_auth,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter(
			EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::from("fetcher=info")), // TODO: that doesn't look right. Isn't there a better way to use info by default?
		)
		.without_time()
		.init();

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
	// let tasks = settings::config::tasks::get(settings::data::settings())?
	// .into_iter()
	// .map(|named_task| {
	// 	tracing::debug!("Found task: {:?}", named_task.path);
	// 	let templates: config::TemplatesField = Figment::new()
	// 		.merge(Yaml::string(&named_task.task))
	// 		.extract()
	// 		.map_err(|e| Error::InvalidConfig(e, path.clone()))?;

	// 	let mut conf = Figment::new();

	// 	if let Some(templates) = templates.templates {
	// 		for tmpl_path in templates {
	// 			let (tmpl, tmpl_full_path) = settings::config::template(&tmpl_path)?;

	// 			tracing::debug!("Using template: {:?}", tmpl_full_path);

	// 			conf = conf.merge(Yaml::string(&tmpl));
	// 		}
	// 	}

	// 	let task: config::Task = conf
	// 		.merge(Yaml::string(&contents))
	// 		.extract()
	// 		.map_err(|e| Error::InvalidConfig(e, path.clone()))?;

	// 	Ok((
	// 		path.file_stem()
	// 			.expect("Somehow the config file found before wasn't an actual config file after all...")
	// 			.to_str()
	// 			.expect("Config file name isn't a valid unicode")
	// 			.to_string(),
	// 		task.parse(&path)?,
	// 	))
	// })
	// .filter(|task_res| {
	// 	// ignore the task only if it's not an error and is marked as disabled
	// 	task_res
	// 		.as_ref()
	// 		.map(|(_, task)| !task.disabled)
	// 		.unwrap_or(true)
	// })
	// .collect::<Result<Tasks>>()?;

	let tasks = settings::config::tasks::get(&DataSettings {
		twitter_auth: settings::data::twitter()?,
		google_oauth2: settings::data::google_oauth2()?,
		google_password: settings::data::google_password()?,
		telegram: settings::data::telegram()?,
	})?;

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
	let mut running_tasks = Vec::new();
	for NamedTask {
		name,
		path: _,
		task: mut t,
	} in tasks
	{
		// if t.disabled {
		// 	continue;
		// }

		let mut shutdown_rx = shutdown_rx.clone();

		let fut = tokio::spawn(async move {
			let mut read_filter = match settings::read_filter::get(name.clone())? {
				f @ Some(_) => f,
				None => t
					.read_filter_kind()
					.map(|k| ReadFilter::new(k, name.clone())),
			};
			let mut save_file = settings::read_filter::save_file(&name)?;

			async {
				select! {
					res = run_task(&name, &mut t, read_filter.as_mut(), &mut save_file) => {
						if let Err(e) = res {
							// TODO: temporary, move that to a tracing layer that sends all WARN and higher logs automatically
							use fetcher::sink::Telegram;
							use fetcher::sink::Message;

							let err_str = format!("{:?}", anyhow::anyhow!(e));
							tracing::error!("{}", err_str);
							if !cfg!(debug_assertions) {
								let send_job = async {
									let bot = match settings::data::telegram()? {
										Some(b) => b,
										None => {
											let s = "Unable to send error report to the admin: telegram bot token is not provided".to_owned();
											tracing::error!(%s);
											return Err(Error::Other(s));	// TODO: this kinda sucks
										}
									};
									let msg = Message {
										body: err_str,
										..Default::default()
									};
									Telegram::new(bot, std::env!("FETCHER_DEBUG_ADMIN_CHAT_ID").to_owned()).send(msg, Some(&name)).await?;
									Ok::<(), Error>(())
								};
								if let Err(e) = send_job.await {
									tracing::error!("Unable to send error report to the admin: {:?}", anyhow::anyhow!(e));
								}
							}
						}
					}
					_ = shutdown_rx.changed() => (),
				}

				tracing::info!("Shutting down...");
				#[allow(unreachable_code)]
				Ok::<(), Error>(())
			}
			.instrument(tracing::info_span!("task", name = name.as_str()))
			.await
		});

		running_tasks.push(flatten_task(fut));
	}

	let _ = join_all(running_tasks).await;
	Ok(())
}

async fn flatten_task<T>(h: JoinHandle<Result<T>>) -> Result<T> {
	match h.await {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(err)) => Err(err),
		e => e.unwrap(), // unwrap NOTE: crash (for now) if there was an error joining the thread
	}
}
