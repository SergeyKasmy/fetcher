pub fn init() {
	set_up_logging();
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
