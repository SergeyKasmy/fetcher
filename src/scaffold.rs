use std::process;

use tokio::sync::watch;

use crate::ctrl_c_signal::CtrlCSignalChannel;

pub struct InitResult {
	pub ctrl_c_signal_channel: CtrlCSignalChannel,
}

#[must_use = "ctrl_c_signal_channel should probably be used. Ignore this type manually if you are sure you don't want it"]
pub fn init() -> InitResult {
	set_up_logging();
	let ctrlc_chan = set_up_ctrl_c_handler();

	InitResult {
		ctrl_c_signal_channel: ctrlc_chan,
	}
}

pub fn set_up_logging() {
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
}

pub fn set_up_ctrl_c_handler() -> CtrlCSignalChannel {
	let (shutdown_tx, shutdown_rx) = watch::channel(());

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

		tracing::info!("Force closing...");
		process::exit(1);
	});

	CtrlCSignalChannel(shutdown_rx)
}
