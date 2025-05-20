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

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> (Vec<JobRunResult>, Self) {
		let (g1_res, g2_res) = join!(self.0.run_in_parallel(), self.1.run_in_parallel());

		let this = Self(g1_res.1, g2_res.1);
		let results = g1_res.0.into_iter().chain(g2_res.0.into_iter()).collect();

		(results, this)
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		self.0.names().chain(self.1.names())
	}
}
