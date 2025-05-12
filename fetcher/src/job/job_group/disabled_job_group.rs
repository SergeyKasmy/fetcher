use super::JobGroup;

pub struct DisabledJobGroup<G>(pub G);

impl<G> JobGroup for DisabledJobGroup<G>
where
	G: JobGroup,
{
	async fn run_concurrently(&mut self) -> Vec<super::JobRunResult> {
		Vec::new()
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		self.0.names()
	}
}
