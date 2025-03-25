use super::JobGroup;

pub struct DisabledJobGroup<J>(pub J);

impl<J> JobGroup for DisabledJobGroup<J> {
	async fn run_concurrently(&mut self) -> Vec<super::JobRunResult> {
		Vec::new()
	}
}
