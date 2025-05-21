use super::JobGroup;
use crate::job::JobResult;

pub struct DisabledJobGroup<G>(pub G);

impl<G> JobGroup for DisabledJobGroup<G>
where
	G: JobGroup,
{
	async fn run_concurrently(&mut self) -> Vec<JobResult> {
		Vec::new()
	}

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> super::MultithreadedJobGroupResult<Self> {
		super::MultithreadedJobGroupResult::JobsFinished {
			job_results: Vec::new(),
			this: self,
		}
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		self.0.names()
	}
}
