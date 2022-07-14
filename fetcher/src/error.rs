pub(crate) mod config;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Config error")]
	Config(#[from] config::Error),

	#[error(transparent)]
	FetcherCoreError(#[from] fetcher_core::error::Error),
}
