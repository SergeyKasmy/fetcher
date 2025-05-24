use std::convert::Infallible;

use crate::maybe_send::{MaybeSend, MaybeSendSync};

use super::JobResult;

pub trait OpaqueJob: MaybeSendSync {
	fn run(&mut self) -> impl Future<Output = JobResult> + MaybeSend;

	fn name(&self) -> Option<&str> {
		None
	}
}

impl OpaqueJob for () {
	async fn run(&mut self) -> JobResult {
		JobResult::Ok
	}
}

impl<J> OpaqueJob for Option<J>
where
	J: OpaqueJob,
{
	async fn run(&mut self) -> JobResult {
		let Some(job) = self else {
			return JobResult::Ok;
		};

		job.run().await
	}

	fn name(&self) -> Option<&str> {
		self.as_ref().and_then(|x| x.name())
	}
}

impl OpaqueJob for Infallible {
	async fn run(&mut self) -> JobResult {
		unreachable!()
	}
}
