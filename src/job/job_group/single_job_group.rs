use std::iter;

use crate::job::OpaqueJob;

use super::{JobGroup, JobResult};

pub struct SingleJobGroup<J>(pub J);

#[cfg(feature = "multithreaded")]
impl<J> JobGroup for SingleJobGroup<J>
where
	J: OpaqueJob + 'static,
{
	async fn run_concurrently(&mut self) -> Vec<JobResult> {
		vec![OpaqueJob::run(&mut self.0).await]
	}

	async fn run_in_parallel(self) -> super::MultithreadedJobGroupResult<Self> {
		use super::MultithreadedJobGroupResult as Res;

		let task_result = tokio::spawn(async move {
			let mut this = self;
			let result = OpaqueJob::run(&mut this.0).await;
			(result, this)
		})
		.await;

		match task_result {
			Ok((result, this)) => Res::JobsFinished {
				job_results: vec![result],
				this,
			},
			Err(join_error) => Res::JobPanicked(join_error),
		}
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		iter::once(self.0.name())
	}
}

#[cfg(not(feature = "multithreaded"))]
impl<J> JobGroup for SingleJobGroup<J>
where
	J: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		vec![OpaqueJob::run(&mut self.0).await]
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		iter::once(self.0.name())
	}
}
