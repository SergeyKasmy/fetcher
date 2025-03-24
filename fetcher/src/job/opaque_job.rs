use std::convert::Infallible;

use crate::{ctrl_c_signal::CtrlCSignalChannel, error::FetcherError};

pub trait OpaqueJob {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>>;

	async fn run_interruptible(
		&mut self,
		ctrl_c_signal_channel: CtrlCSignalChannel,
	) -> Result<(), Vec<FetcherError>>;

	fn name(&self) -> Option<&str> {
		None
	}
}

impl OpaqueJob for () {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		Ok(())
	}

	async fn run_interruptible(
		&mut self,
		_ctrl_c_signal_channel: CtrlCSignalChannel,
	) -> Result<(), Vec<FetcherError>> {
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

	async fn run_interruptible(
		&mut self,
		ctrl_c_signal_channel: CtrlCSignalChannel,
	) -> Result<(), Vec<FetcherError>> {
		let Some(job) = self else {
			return Ok(());
		};

		job.run_interruptible(ctrl_c_signal_channel).await
	}

	fn name(&self) -> Option<&str> {
		self.as_ref().and_then(|x| x.name())
	}
}

impl OpaqueJob for Infallible {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		unreachable!()
	}

	async fn run_interruptible(
		&mut self,
		_ctrl_c_signal_channel: CtrlCSignalChannel,
	) -> Result<(), Vec<FetcherError>> {
		unreachable!()
	}
}
