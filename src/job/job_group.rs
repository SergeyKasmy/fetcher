mod combined_job_group;
mod disabled_job_group;
mod named_job_group;

use std::iter;
use tokio::join;

use super::{JobResult, OpaqueJob};
use crate::StaticStr;
use crate::maybe_send::{MaybeSend, MaybeSendSync};

pub use self::combined_job_group::CombinedJobGroup;
pub use self::disabled_job_group::DisabledJobGroup;
pub use self::named_job_group::NamedJobGroup;

pub type JobGroupResult = Vec<JobResult>;

pub trait JobGroup: MaybeSendSync {
	#[must_use = "the jobs could've finished with errors"]
	fn run_concurrently(&mut self) -> impl Future<Output = JobGroupResult> + MaybeSend;

	#[cfg(feature = "multithreaded")]
	#[must_use = "the jobs could've finished with errors"]
	fn run_in_parallel(self) -> impl Future<Output = (JobGroupResult, Self)> + Send
	where
		Self: Sized + 'static;

	fn names(&self) -> impl Iterator<Item = Option<String>>;

	#[cfg(feature = "multithreaded")]
	#[must_use = "the jobs could've finished with errors"]
	fn run(self) -> impl Future<Output = (JobGroupResult, Self)> + MaybeSend
	where
		Self: Sized + 'static,
	{
		async move { self.run_in_parallel().await }
	}

	#[cfg(not(feature = "multithreaded"))]
	#[must_use = "the jobs could've finished with errors"]
	fn run(mut self) -> impl Future<Output = (JobGroupResult, Self)> + MaybeSend
	where
		Self: Sized,
	{
		async move { (self.run_concurrently().await, self) }
	}

	fn combine_with<G>(self, other: G) -> CombinedJobGroup<Self, G>
	where
		Self: Sized,
		G: JobGroup,
	{
		CombinedJobGroup(self, other)
	}

	fn disable(self) -> DisabledJobGroup<Self>
	where
		Self: Sized,
	{
		DisabledJobGroup(self)
	}

	fn with_name<S>(self, name: S) -> NamedJobGroup<Self>
	where
		Self: Sized,
		S: Into<StaticStr>,
	{
		NamedJobGroup {
			inner: self,
			name: name.into(),
		}
	}
}

impl<J> JobGroup for Option<J>
where
	J: JobGroup,
{
	async fn run_concurrently(&mut self) -> JobGroupResult {
		let Some(group) = self else {
			return Vec::new();
		};

		group.run_concurrently().await
	}

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> (JobGroupResult, Self)
	where
		Self: 'static,
	{
		let Some(group) = self else {
			return (Vec::new(), None);
		};

		let (job_results, inner) = group.run_in_parallel().await;
		(job_results, Some(inner))
	}

	fn names(&self) -> impl Iterator<Item = Option<String>> {
		self.iter().flat_map(JobGroup::names)
	}
}

impl JobGroup for () {
	async fn run_concurrently(&mut self) -> JobGroupResult {
		Vec::new()
	}

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> (JobGroupResult, Self) {
		(Vec::new(), ())
	}

	fn names(&self) -> impl Iterator<Item = Option<String>> {
		iter::empty()
	}
}

impl<G> JobGroup for (G,)
where
	G: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> JobGroupResult {
		vec![self.0.run().await]
	}

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> (JobGroupResult, Self)
	where
		Self: 'static,
	{
		let (result, inner) = run_job_in_parallel(self.0).await;
		(vec![result], (inner,))
	}

	fn names(&self) -> impl Iterator<Item = Option<String>> {
		iter::once(self.0.name().map(ToOwned::to_owned))
	}
}

macro_rules! impl_jobgroup_for_tuples {
	($($type_name:ident)+) => {
		impl<$($type_name),+> JobGroup for ($($type_name),+)
		where
			$($type_name: OpaqueJob),+
		{
			#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
			async fn run_concurrently(&mut self) -> JobGroupResult {
				// $type_name = specific job
				let ($($type_name),+) = self;

				// $type_name = $type_name's job result
				let ($($type_name),+) = join!( $($type_name.run()),+ );

				// destructure the tuple into an array and then convert it
				JobGroupResult::from([$($type_name),+])
			}

			#[cfg(feature = "multithreaded")]
			#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
			async fn run_in_parallel(self) -> (JobGroupResult, Self)
			where Self: 'static
			{
				// $type_name = specific job
				let ($($type_name),+) = self;

				// $type_name = typle of (job_result, old $type_name)
				let ($($type_name),+) = join!($(run_job_in_parallel($type_name)),+);

				// destructure the tuple into an array and then convert it
				let results = JobGroupResult::from([$($type_name.0),+]);
				let this = ($($type_name.1),+);

				(results, this)
			}

			#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
			fn names(&self) -> impl Iterator<Item = Option<String>> {
				let ($($type_name),+) = self;

				iter::empty()
					$(
						.chain(
							iter::once(
								$type_name.name().map(ToOwned::to_owned)
							)
						)
					)+
			}
		}
	}
}

impl_jobgroup_for_tuples!(J1 J2);
impl_jobgroup_for_tuples!(J1 J2 J3);
impl_jobgroup_for_tuples!(J1 J2 J3 J4);
impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5);
impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6);
impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7);
impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8);
impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9);
impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10);

#[cfg(feature = "multithreaded")]
async fn run_job_in_parallel<J>(mut job: J) -> (JobResult, J)
where
	J: OpaqueJob + 'static,
{
	use tracing::Instrument;

	let async_task = async move {
		let result = OpaqueJob::run(&mut job).await;
		(result, job)
	};

	tokio::spawn(async_task.in_current_span())
		.await
		.expect("should never panic, all panicked should've been caught")
}
