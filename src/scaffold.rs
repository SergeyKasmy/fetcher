use std::process;

use tokio::sync::watch;
use tracing::subscriber::SetGlobalDefaultError;

use crate::ctrl_c_signal::CtrlCSignalChannel;

pub struct InitResult {
	pub ctrl_c_signal_channel: CtrlCSignalChannel,
}

#[must_use = "ctrl_c_signal_channel should probably be used. Ignore this type manually if you are sure you don't want it"]
pub fn init() -> Result<InitResult, SetGlobalDefaultError> {
	set_up_logging()?;

	if tokio_rustls::rustls::crypto::aws_lc_rs::default_provider()
		.install_default()
		.is_err()
	{
		tracing::debug!("Unable to set up aws_lc as the default crypto provider for rustls");
	}

	let ctrlc_chan = set_up_ctrl_c_handler();

	Ok(InitResult {
		ctrl_c_signal_channel: ctrlc_chan,
	})
}

/// Installs a tracing subscriber as the default.
///
/// The subscriber shows compact one-line log messages when log level is > DEBUG,
/// and pretty multi-line log messages when it's set to <= DEBUG.
///
/// It also logs to systemd-journald if available but only when compiled in release (to avoid log spam when debugging)
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
