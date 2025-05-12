mod combined_job_group;
mod disabled_job_group;
mod single_job_group;

use std::iter;
use tokio::join;

use self::combined_job_group::CombinedJobGroup;
use crate::error::FetcherError;

pub(crate) use self::single_job_group::SingleJobGroup;

pub use self::disabled_job_group::DisabledJobGroup;

use super::OpaqueJob;

pub type JobRunResult = Result<(), Vec<FetcherError>>;

pub trait JobGroup {
	#[must_use = "this vec of results could contain errors"]
	async fn run_concurrently(&mut self) -> Vec<JobRunResult>;

	fn names(&self) -> impl Iterator<Item = Option<&str>>;

	fn and<J>(self, other: J) -> CombinedJobGroup<Self, impl JobGroup>
	where
		Self: Sized,
		J: OpaqueJob,
	{
		self.with(SingleJobGroup(other))
	}

	fn with<G>(self, other: G) -> CombinedJobGroup<Self, G>
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
}

impl<J> JobGroup for Option<J>
where
	J: JobGroup,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let Some(group) = self else {
			return Vec::new();
		};

		group.run_concurrently().await
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		self.iter().flat_map(|j| j.names())
	}
}

impl<J1> JobGroup for (J1,)
where
	J1: JobGroup,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		self.0.run_concurrently().await
	}

	fn names(&self) -> impl Iterator<Item = Option<&str>> {
		self.0.names()
	}
}

macro_rules! impl_jobgroup_for_tuples {
	($($type_name:ident)+) => {
		impl<$($type_name),+> JobGroup for ($($type_name),+)
		where
			$($type_name: JobGroup),+
		{
			async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
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

			fn names(&self) -> impl Iterator<Item = Option<&str>> {
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
