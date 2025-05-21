use std::iter;

use crate::job::OpaqueJob;

use super::{JobGroup, JobGroupResult};

pub struct SingleJobGroup<J>(pub J);

#[cfg(feature = "multithreaded")]
impl<J> JobGroup for SingleJobGroup<J>
where
	J: OpaqueJob + 'static,
{
	async fn run_concurrently(&mut self) -> JobGroupResult {
		vec![OpaqueJob::run(&mut self.0).await]
	}

	async fn run_in_parallel(self) -> (JobGroupResult, Self) {
		tokio::spawn(async move {
			let mut this = self;
			let result = OpaqueJob::run(&mut this.0).await;
			(vec![result], this)
		})
		.await
		.expect("should never panic, all panicked should've been caught")
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
	async fn run_concurrently(&mut self) -> JobGroupResult {
		vec![OpaqueJob::run(&mut self.0).await]
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		iter::once(self.0.name())
	}
}
