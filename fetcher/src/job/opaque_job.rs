use std::convert::Infallible;

use crate::error::FetcherError;

pub trait OpaqueJob {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>>;

	fn name(&self) -> Option<&str> {
		None
	}
}

impl OpaqueJob for () {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		Ok(())
	}
}

impl<J> OpaqueJob for Option<J>
where
	J: OpaqueJob,
{
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		let Some(job) = self else {
			return Ok(());
		};

		job.run().await
	}

	fn name(&self) -> Option<&str> {
		self.as_ref().and_then(|x| x.name())
	}
}

impl OpaqueJob for Infallible {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		unreachable!()
	}
}
