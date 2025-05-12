use crate::job::OpaqueJob;

use super::JobGroup;

pub struct SingleJobGroup<J>(pub J);

impl<J> JobGroup for SingleJobGroup<J>
where
	J: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<super::JobRunResult> {
		vec![OpaqueJob::run(&mut self.0).await]
	}
}
