use super::{JobGroup, JobGroupResult};

pub struct DisabledJobGroup<G>(pub G);

impl<G> JobGroup for DisabledJobGroup<G>
where
	G: JobGroup,
{
	async fn run_concurrently(&mut self) -> JobGroupResult {
		Vec::new()
	}

	#[cfg(feature = "send")]
	async fn run_in_parallel(self) -> (JobGroupResult, Self) {
		(Vec::new(), self)
	}

	fn names(&self) -> impl Iterator<Item = Option<String>> {
		self.0.names()
	}
}
