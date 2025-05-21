use futures::join;

use super::{JobGroup, JobGroupResult};

pub struct CombinedJobGroup<G1, G2>(pub G1, pub G2);

impl<G1, G2> JobGroup for CombinedJobGroup<G1, G2>
where
	G1: JobGroup,
	G2: JobGroup,
{
	async fn run_concurrently(&mut self) -> JobGroupResult {
		let results = join!(self.0.run_concurrently(), self.1.run_concurrently());

		results.0.into_iter().chain(results.1.into_iter()).collect()
	}

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> (JobGroupResult, Self) {
		let ((job_results1, inner1), (job_results2, inner2)) =
			join!(self.0.run_in_parallel(), self.1.run_in_parallel());

		let job_results = job_results1
			.into_iter()
			.chain(job_results2.into_iter())
			.collect();

		let this = Self(inner1, inner2);
		(job_results, this)
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		self.0.names().chain(self.1.names())
	}
}
