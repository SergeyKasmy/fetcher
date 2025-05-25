/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains a "scaffold", in other words, functions that pre-configure your application for common uses of [`fetcher`](`crate`).
//!
//! The main entry point of this module is [`init`]

use std::process;

use tokio::sync::watch;
use tracing::subscriber::SetGlobalDefaultError;

use crate::ctrl_c_signal::CtrlCSignalChannel;

/// Contains the result of the [`init`] function
#[must_use = "ctrl_c_signal_channel should probably be used. Ignore this type manually if you are sure you don't want it"]
pub struct InitResult {
	/// The receiving channel end of the Ctrl-C signal handler background task
	pub ctrl_c_signal_channel: CtrlCSignalChannel,
}

/// Initializes a tracing subscriber and a background task that will notify when a Ctrl-C signal has arrived via [`CtrlCSignalChannel`].
///
/// See [`set_up_logging`] and [`set_up_ctrl_c_handler`] for more info
pub fn init() -> InitResult {
	if set_up_logging().is_err() {
		tracing::debug!(
			"Unable to set up the default tracing subscriber. Another one is probably already registered"
		);
	}

	if tokio_rustls::rustls::crypto::aws_lc_rs::default_provider()
		.install_default()
		.is_err()
	{
		tracing::debug!("Unable to set up aws_lc as the default crypto provider for rustls");
	}

	InitResult {
		ctrl_c_signal_channel: set_up_ctrl_c_handler(),
	}
}

/// Installs a tracing subscriber as the default.
///
/// The subscriber shows compact one-line log messages when log level is > DEBUG,
/// and pretty multi-line log messages when it's set to <= DEBUG.
///
/// It also logs to systemd-journald if available but only when compiled in release (to avoid log spam when debugging)
///
/// # Errors
/// If a different global tracing subscriber has already been registered.
///
/// # Panics
/// If local timezone wasn't able to be determined. This should never happen and if it does, please submit a bug report.  
pub fn set_up_logging() -> Result<(), SetGlobalDefaultError> {
	use tracing::Level;
	use tracing_subscriber::{
		EnvFilter, Layer, filter::LevelFilter, fmt::time::OffsetTime, layer::SubscriberExt,
	};

	let env_filter = EnvFilter::builder()
		.with_default_directive(LevelFilter::INFO.into())
		.from_env_lossy();

	let is_debug_log_level = env_filter
		.max_level_hint()
		.map_or_else(|| false, |level| level >= Level::DEBUG);

	let stdout = tracing_subscriber::fmt::layer()
		.with_target(is_debug_log_level)
		.with_file(is_debug_log_level)
		.with_line_number(is_debug_log_level)
		.with_thread_ids(is_debug_log_level)
		// TODO: may panic in multithreaded env, including when running in a tokio runtime.
		// For now this has never happened but if it does,
		// we should probably do something about it (e.g. move this call to before the tokio runtime is initialized)
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
}

/// Starts a detached tokio::task that sets up a Ctrl-C signal handler
///
/// When a Ctrl-C signal is received,
/// all jobs waiting on the [`CtrlCSignalChannel`] will receive a notification
/// and attempt to shutdown.
#[must_use]
pub fn set_up_ctrl_c_handler() -> CtrlCSignalChannel {
	let (shutdown_tx, shutdown_rx) = watch::channel(());

	// signal handler
	tokio::spawn(async move {
		// graceful shutdown
		if let Err(e) = tokio::signal::ctrl_c().await {
			tracing::error!("Failed to set up a CtrlC signal handler: {e}");
			return;
		}

		// shutdown signal recieved
		_ = shutdown_tx.send(());

		tracing::info!("Press Ctrl-C again to force close");

		// force close
		if let Err(e) = tokio::signal::ctrl_c().await {
			tracing::error!("Failed to set up a CtrlC signal handler: {e}");
			return;
		}

		tracing::info!("Force closing...");
		#[expect(clippy::exit, reason = "user requested force close")]
		process::exit(1);
	});

	CtrlCSignalChannel(shutdown_rx)
}
