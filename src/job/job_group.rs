/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`JobGroup`] trait that is used for running multiple jobs together.

mod combined_job_group;
mod disabled_job_group;
mod named_job_group;

use futures::{
	Stream,
	future::Either as FutureEither,
	stream::{self, FuturesUnordered},
};
use tokio::join;

use std::iter;
use std::pin::Pin;

use super::{JobResult, OpaqueJob};
use crate::StaticStr;
use crate::maybe_send::{MaybeSend, MaybeSendSync};

pub use self::combined_job_group::CombinedJobGroup;
pub use self::disabled_job_group::DisabledJobGroup;
pub use self::named_job_group::NamedJobGroup;

/// Result type returned by job groups containing results from all jobs in the group.
// TODO: make a generic?
pub type JobGroupResult = Vec<JobResult>;

/// A group of jobs that can be run together.
///
/// This trait provides functionality to:
/// - Run multiple jobs concurrently or in parallel
/// - Combine job groups together
/// - Disable groups temporarily
/// - Add names to groups
///
/// Job groups can be created in several ways:
/// 1. Using tuples of jobs
/// 2. Combining existing groups with [`combine_with`](JobGroup::combine_with)
/// 3. Using [`Option`] for optional groups
/// 4. Using unit `()` for empty groups
///
/// # Running Jobs
/// Jobs in a group can be run in two ways:
///
/// - [`run_concurrently`](JobGroup::run_concurrently): Runs jobs concurrently in the same async task
/// - [`run_in_parallel`](JobGroup::run_in_parallel): Spawns each job on a separate task (requires `send` feature)
///
/// The [`run`](JobGroup::run) method automatically chooses between these based on the `send` feature flag.
///
/// # Example
/// ```rust
/// # tokio_test::block_on(async {
/// use fetcher::job::{Job, JobGroup, error_handling::Forward, RefreshTime};
///
/// // Create jobs
/// let job1 = Job::builder("job1")
///                 .tasks(())
///                 .refresh_time(RefreshTime::Never)
///                 .error_handling(Forward)
///                 .ctrlc_chan(None)
///                 .build();
/// let job2 = Job::builder("job2")
///                 .tasks(())
///                 .refresh_time(RefreshTime::Never)
///                 .error_handling(Forward)
///                 .ctrlc_chan(None)
///                 .build();
///
/// // Group jobs using a tuple
/// let mut group = (job1, job2);
///
/// // Run jobs and get results
/// //let results = group.run_concurrently().await;
///
/// // Add a name to the group
/// let named_group = group.with_name("my_group");
///
/// // Temporarily disable the group
/// let disabled = named_group.disable();
/// # });
/// ```
pub trait JobGroup: MaybeSendSync {
	/// Runs all jobs in the group concurrently in the same async task.
	///
	/// This method runs all jobs in the group concurrently using [`join!()`],
	/// but does not spawn new tasks. All jobs run in the same async task.
	/// This is in contrast to [`JobGroup::run_in_parallel`].
	#[must_use = "the jobs could've finished with errors"]
	fn run_concurrently(&mut self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend;

	/// Runs all jobs in the group in parallel on separate tasks.
	///
	/// This method spawns each job on a separate task using `tokio::spawn`.
	/// Only available when the `send` feature is enabled.
	/// This is in contrast to [`JobGroup::run_concurrently`].
	#[cfg(feature = "send")]
	#[must_use = "the jobs could've finished with errors"]
	fn run_in_parallel(self) -> impl Future<Output = (JobGroupResult, Self)> + Send
	where
		Self: Sized + 'static;

	/// Returns the names of all jobs in the group.
	///
	/// Returns `None` for unnamed jobs.
	fn names(&self) -> impl Iterator<Item = Option<String>>;

	/// Run all jobs in the group using the most appropriate method.
	///
	/// When the `send` feature is enabled, this will run jobs in parallel using [`run_in_parallel`](JobGroup::run_in_parallel).
	/// Otherwise, it will run jobs concurrently using [`run_concurrently`](JobGroup::run_concurrently).
	///
	/// This is the recommended way to run jobs unless you specifically need concurrent or parallel execution.
	#[cfg(feature = "send")]
	#[must_use = "the jobs could've finished with errors"]
	// TODO: return a stream of job results instead!
	// Makes it possible to somehow handle errors and panics in jobs without waiting for other jobs to stop
	fn run(self) -> impl Future<Output = (JobGroupResult, Self)> + MaybeSend
	where
		Self: Sized + 'static,
	{
		async move { self.run_in_parallel().await }
	}

	/// Run all jobs in the group using the most appropriate method.
	///
	/// When the `send` feature is enabled, this will run jobs in parallel using [`run_in_parallel`](JobGroup::run_in_parallel).
	/// Otherwise, it will run jobs concurrently using [`run_concurrently`](JobGroup::run_concurrently).
	///
	/// This is the recommended way to run jobs unless you specifically need concurrent or parallel execution.
	#[cfg(not(feature = "send"))]
	#[must_use = "the jobs could've finished with errors"]
	fn run(mut self) -> impl Future<Output = (JobGroupResult, Self)> + MaybeSend
	where
		Self: Sized,
	{
		// async move { (self.run_concurrently().await, self) }
		todo!()
	}

	/// Combine this job group with another job group.
	///
	/// Creates a new [`CombinedJobGroup`] that will run all jobs from both grorups.
	///
	/// # Example
	/// ```rust
	/// # tokio_test::block_on(async {
	/// use fetcher::job::{Job, JobGroup, error_handling::Forward, RefreshTime};
	///
	/// // Create jobs
	/// let job1 = Job::builder("job1").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	/// let job2 = Job::builder("job2").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	/// let job3 = Job::builder("job3").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	/// let job4 = Job::builder("job4").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	///
	/// let group1 = (job1, job2);
	/// let group2 = (job3, job4);
	//r/
	//r/ // Combine the groups
	/// let combined = group1.combine_with(group2);
	///
	/// let (results, _combined) = combined.run().await;
	/// # });
	/// ```
	fn combine_with<G>(self, other: G) -> CombinedJobGroup<Self, G>
	where
		Self: Sized,
		G: JobGroup,
	{
		CombinedJobGroup(self, other)
	}

	/// Temporarily disable this job group.
	///
	/// Creates a new [`DisabledJobGroup`] that wraps thris group but does not execute any jobs.
	/// The group can be re-enabled by unwrapping the [`DisabledJobGroup`].
	///
	/// This is useful for temporarily disabling a set of jobs without removing them from the code.
	///
	/// # Example
	/// ```rust
	/// # tokio_test::block_on(async {
	/// use fetcher::job::{Job, JobGroup, error_handling::Forward, RefreshTime};
	///
	/// // Create jobs
	/// let job1 = Job::builder("job1").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	/// let job2 = Job::builder("job2").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	///
	/// let group = (job1, job2);
	///
	/// // Disable the group
	/// let disabled = group.disable();
	///
	/// // Running the disabled group will do nothing
	/// let (results, _) = disabled.run().await;
	/// assert!(results.is_empty());
	/// # });
	/// ```
	fn disable(self) -> DisabledJobGroup<Self>
	where
		Self: Sized,
	{
		DisabledJobGroup(self)
	}

	/// Add a name to this job group.
	///
	/// Creates a new [`NamedJobGroup`] that wraps this group with the given name.
	/// The name is used for logging and debugging purposes.
	///
	/// # Example
	/// ```rust
	/// # tokio_test::block_on(async {
	/// use fetcher::job::{Job, JobGroup, error_handling::Forward, RefreshTime};
	///
	/// let job1 = Job::builder("job1").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	/// let job2 = Job::builder("job2").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	///
	/// let group = (job1, job2);
	///
	/// // Add a name to the group
	/// let named_group = group.with_name("important_jobs");
	///
	/// let names = named_group.names().flatten().collect::<Vec<String>>();
	/// assert_eq!(names, vec!["important_jobs/job1", "important_jobs/job2"]);
	///
	/// // Add a second name to the group
	/// let named_group2 = named_group.with_name("something_else");
	///
	/// let names = named_group2.names().flatten().collect::<Vec<String>>();
	/// assert_eq!(names, vec!["something_else/important_jobs/job1", "something_else/important_jobs/job2"]);
	/// # });
	/// ```
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

// FIXME: REMOVEME DON'T DERIVE
#[derive(Default)]
pub struct JobId {
	// outer to inner
	pub group_hierarchy: Vec<StaticStr>,
	pub job_name: Option<StaticStr>,
}

impl<J> JobGroup for Option<J>
where
	J: JobGroup,
{
	// async fn run_concurrently(&mut self) -> JobGroupResult {
	// 	let Some(group) = self else {
	// 		return Vec::new();
	// 	};

	// 	group.run_concurrently().await
	// }

	fn run_concurrently(&mut self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend {
		let Some(group) = self else {
			return FutureEither::Left(stream::empty());
		};

		FutureEither::Right(group.run_concurrently())
	}

	#[cfg(feature = "send")]
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
	fn run_concurrently(&mut self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend {
		stream::empty()
	}

	#[cfg(feature = "send")]
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
	fn run_concurrently(&mut self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend {
		stream::once(async move { (JobId::default(), self.0.run().await) })
	}

	#[cfg(feature = "send")]
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
			fn run_concurrently(&mut self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend {
				#[cfg(feature = "send")]
				type MaybeSendBoxedFuture<'a> = Pin<Box<dyn Future<Output = (JobId, JobResult)> + Send + 'a>>;
				#[cfg(not(feature = "send"))]
				type MaybeSendBoxedFuture<'a> = Pin<Box<dyn Future<Output = (JobId, JobResult)> + 'a>>;

				/// Runs the job, attaches a Job::Id and boxes the resulting future
				fn into_maybe_send_boxed_future<'a, J: OpaqueJob>(job: &'a mut J) -> MaybeSendBoxedFuture<'a> {
					let attach_id_to_result_fut = async move {
						let job_result = job.run().await;
						(JobId::default(), job_result)
					};

					Box::pin(attach_id_to_result_fut)
				}

				// $type_name = specific job
				let ($($type_name),+) = self;

				[
					$(into_maybe_send_boxed_future($type_name)),+
				]
				.into_iter()
				.collect::<FuturesUnordered<_>>()
			}

			#[cfg(feature = "send")]
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

#[cfg(feature = "send")]
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
