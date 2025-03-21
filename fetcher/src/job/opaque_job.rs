use std::convert::Infallible;

use tokio::select;

use crate::{ctrl_c_signal::CtrlCSignalChannel, error::FetcherError};

pub trait OpaqueJob {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>>;

	fn disable(self) -> DisabledJob<Self>
	where
		Self: Sized,
	{
		DisabledJob(self)
	}

	fn name(&self) -> Option<&str> {
		None
	}

	async fn run_interruptible(
		&mut self,
		mut ctrl_c_signal_channel: CtrlCSignalChannel,
	) -> Result<(), Vec<FetcherError>> {
		select! {
			res = self.run() => {
				res
			}
			_ = ctrl_c_signal_channel.signaled() => {
				if let Some(name) = self.name() {
					tracing::info!("Job {name} signaled to shutdown...");
				}
				Ok(())
			}
		}
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

pub struct DisabledJob<J>(J);

impl<J> OpaqueJob for DisabledJob<J> {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		Ok(())
	}
}

impl OpaqueJob for Infallible {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		unreachable!()
	}
}
