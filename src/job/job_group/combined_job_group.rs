use futures::join;

use super::{JobGroup, JobResult};

pub struct CombinedJobGroup<G1, G2>(pub G1, pub G2);

impl<G1, G2> JobGroup for CombinedJobGroup<G1, G2>
where
	G1: JobGroup,
	G2: JobGroup,
{
	async fn run_concurrently(&mut self) -> Vec<JobResult> {
		let results = join!(self.0.run_concurrently(), self.1.run_concurrently());

		results.0.into_iter().chain(results.1.into_iter()).collect()
	}

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> super::MultithreadedJobGroupResult<Self> {
		use crate::try_jg_res;

		let (g1_res, g2_res) = join!(self.0.run_in_parallel(), self.1.run_in_parallel());

		let (job_results1, inner1) = try_jg_res!(g1_res);
		let (job_results2, inner2) = try_jg_res!(g2_res);

		let job_results = job_results1
			.into_iter()
			.chain(job_results2.into_iter())
			.collect();

		let this = Self(inner1, inner2);

		super::MultithreadedJobGroupResult::JobsFinished { job_results, this }
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		self.0.names().chain(self.1.names())
	}
}
