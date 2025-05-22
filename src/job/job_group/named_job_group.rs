use crate::StaticStr;

use super::JobGroup;

pub struct NamedJobGroup<G> {
	pub inner: G,
	pub name: StaticStr,
}

impl<G> JobGroup for NamedJobGroup<G>
where
	G: JobGroup,
{
	async fn run_concurrently(&mut self) -> super::JobGroupResult {
		self.inner.run_concurrently().await
	}

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> (super::JobGroupResult, Self)
	where
		Self: Sized,
	{
		let (job_results, inner) = self.inner.run_in_parallel().await;
		(
			job_results,
			Self {
				inner,
				name: self.name,
			},
		)
	}

	fn names(&self) -> impl Iterator<Item = Option<String>> {
		self.inner.names().map(|name| {
			let Some(mut name) = name else {
				return None;
			};

			name.insert(0, '/');
			name.insert_str(0, &self.name);

			Some(name)
		})
	}
}
