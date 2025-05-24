mod combined_job_group;
mod disabled_job_group;
mod named_job_group;
mod single_job_group;

use std::iter;
use tokio::join;

use super::JobResult;
use crate::StaticStr;
use crate::maybe_send::{MaybeSend, MaybeSendSync};

pub use self::combined_job_group::CombinedJobGroup;
pub use self::disabled_job_group::DisabledJobGroup;
pub use self::named_job_group::NamedJobGroup;
pub use self::single_job_group::SingleJobGroup;

pub type JobGroupResult = Vec<JobResult>;

pub trait JobGroup: MaybeSendSync {
	#[must_use = "the jobs could've finished with errors"]
	fn run_concurrently(&mut self) -> impl Future<Output = JobGroupResult> + MaybeSend;

	#[cfg(feature = "multithreaded")]
	#[must_use = "the jobs could've finished with errors"]
	fn run_in_parallel(self) -> impl Future<Output = (JobGroupResult, Self)> + Send
	where
		Self: Sized;

	fn names(&self) -> impl Iterator<Item = Option<String>>;

	#[must_use = "the jobs could've finished with errors"]
	fn run(self) -> impl Future<Output = (JobGroupResult, Self)> + MaybeSend
	where
		Self: Sized,
	{
		#[cfg(feature = "multithreaded")]
		let async_block = async move { self.run_in_parallel().await };

		#[cfg(not(feature = "multithreaded"))]
		let async_block = {
			let mut this = self;
			async move { (this.run_concurrently().await, this) }
		};

		async_block
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
	async fn run_in_parallel(self) -> (JobGroupResult, Self) {
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

impl<J1> JobGroup for (J1,)
where
	J1: JobGroup,
{
	async fn run_concurrently(&mut self) -> JobGroupResult {
		self.0.run_concurrently().await
	}

	#[cfg(feature = "multithreaded")]
	async fn run_in_parallel(self) -> (JobGroupResult, Self) {
		let (job_results, inner) = self.0.run_in_parallel().await;
		(job_results, (inner,))
	}

	fn names(&self) -> impl Iterator<Item = Option<String>> {
		self.0.names()
	}
}

macro_rules! impl_jobgroup_for_tuples {
	($($type_name:ident)+) => {
		impl<$($type_name),+> JobGroup for ($($type_name),+)
		where
			$($type_name: JobGroup),+
		{
			async fn run_concurrently(&mut self) -> JobGroupResult {
				// first $type_name = specific job
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = self;

				// now $type_name = job run result
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = join!($($type_name.run_concurrently()),+);

				let mut results = Vec::new();

				$(
					results.extend($type_name);
				)+

				results
			}

			#[cfg(feature = "multithreaded")]
			async fn run_in_parallel(self) -> (JobGroupResult, Self) {
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = self;

				// now $type_name = job run result
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = join!($($type_name.run_in_parallel()),+);

				let mut results = Vec::new();

				$(
					results.extend($type_name.0);
				)+

				let this = ($($type_name.1),+);
				(results, this)
			}

			fn names(&self) -> impl Iterator<Item = Option<String>> {
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = self;

				iter::empty()
					$(.chain($type_name.names()))+
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
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16 J17);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16 J17 J18);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16 J17 J18 J19);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16 J17 J18 J19 J20);
