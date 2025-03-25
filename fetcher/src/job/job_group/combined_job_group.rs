use futures::join;

use super::{JobGroup, JobRunResult};

pub struct CombinedJobGroup<G1, G2>(pub G1, pub G2);

impl<G1, G2> JobGroup for CombinedJobGroup<G1, G2>
where
	G1: JobGroup,
	G2: JobGroup,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(self.0.run_concurrently(), self.1.run_concurrently());

		results.0.into_iter().chain(results.1.into_iter()).collect()
	}
}
