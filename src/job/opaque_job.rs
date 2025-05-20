use std::convert::Infallible;

use crate::error::FetcherError;
use crate::maybe_send::{MaybeSend, MaybeSendSync};

use super::JobGroup;
use super::job_group::SingleJobGroup;

pub trait OpaqueJob: MaybeSendSync {
	fn run(&mut self) -> impl Future<Output = Result<(), Vec<FetcherError>>> + MaybeSend;

	fn name(&self) -> Option<&str> {
		None
	}

	#[cfg(feature = "send")]
	fn group_with<J>(self, other: J) -> impl JobGroup
	where
		Self: Sized + 'static,
		J: OpaqueJob + Sized + 'static,
	{
		SingleJobGroup(self).and(other)
	}

	#[cfg(not(feature = "send"))]
	fn group_with<J>(self, other: J) -> impl JobGroup
	where
		Self: Sized,
		J: OpaqueJob + Sized,
	{
		SingleJobGroup(self).and(other)
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
