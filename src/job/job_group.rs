/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`JobGroup`] trait that is used for running multiple jobs together.

mod combined_job_group;
mod disabled_job_group;
mod named_job_group;

use futures::{Stream, future::Either as FutureEither, stream};
use itertools::Itertools;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use std::fmt::{self, Display};
use std::iter;

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
/// # tokio::task::LocalSet::new().run_until(async {
/// use fetcher::job::{Job, JobGroup, error_handling::Forward, RefreshTime};
/// use futures::stream::StreamExt;
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
/// let mut group_results = group.clone().run();
/// while let Some(job_result) = group_results.next().await {
///     println!("Job {} finished!", job_result.0);
/// }
/// drop(group_results);
///
/// // Add a name to the group
/// let named_group = group.with_name("my_group");
///
/// // Temporarily disable the group
/// let _disabled = named_group.disable();
/// # }).await;
/// # });
/// ```
pub trait JobGroup: MaybeSendSync {
	// TODO: fix docs
	/// Runs all jobs in the group TODO
	///
	/// This method spawns each job on a separate task using `tokio::spawn`.
	#[must_use = "the jobs could've finished with errors"]
	fn run(self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend
	where
		Self: Sized + 'static;

	/// Combine this job group with another job group.
	///
	/// Creates a new [`CombinedJobGroup`] that will run all jobs from both grorups.
	///
	/// # Example
	/// ```rust
	/// # tokio_test::block_on(async {
	/// # tokio::task::LocalSet::new().run_until(async {
	/// use fetcher::job::{Job, JobGroup, error_handling::Forward, RefreshTime};
	/// use futures::stream::StreamExt;
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
	/// let results = combined.run().collect::<Vec<_>>().await;
	/// # }).await;
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
	/// use futures::stream::StreamExt;
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
	/// let results = disabled.run().collect::<Vec<_>>().await;
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
	///
	// TODO: fix doccomment
	// # Example
	// ```rust
	// # tokio_test::block_on(async {
	// use fetcher::job::{Job, JobGroup, error_handling::Forward, RefreshTime};
	//
	// let job1 = Job::builder("job1").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	// let job2 = Job::builder("job2").tasks(()).refresh_time(RefreshTime::Never).error_handling(Forward).ctrlc_chan(None).build();
	//
	// let group = (job1, job2);
	//
	// // Add a name to the group
	// let named_group = group.with_name("important_jobs");
	//
	// let names = named_group.names().flatten().collect::<Vec<String>>();
	// assert_eq!(names, vec!["important_jobs/job1", "important_jobs/job2"]);
	//
	// // Add a second name to the group
	// let named_group2 = named_group.with_name("something_else");
	//
	// let names = named_group2.names().flatten().collect::<Vec<String>>();
	// assert_eq!(names, vec!["something_else/important_jobs/job1", "something_else/important_jobs/job2"]);
	// # });
	// ```
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

/// Unique ID of a job
///
/// Includes its name and all named groups' names.
///
/// Implements [`Display`] to format the name in a filesystem path-like manner
///
/// # Example
/// ```
/// # tokio_test::block_on(async {
/// use fetcher::job::{Job, JobGroup, RefreshTime};
/// use futures::stream::StreamExt;
/// use std::pin::pin;
///
/// let job = Job::builder("job")
/// 				.tasks(())
///                 .refresh_time(RefreshTime::Never)
///                 .ctrlc_chan(None)
///                 .build_with_default_error_handling();
///
/// let inner_group = (job,).with_name("inner group");
/// let mut outer_group = inner_group.with_name("outer group");
/// let mut run_stream = pin!(outer_group.run());
///
/// let (job_id, _job_result) = run_stream.next().await.unwrap();
/// assert_eq!(job_id.to_string(), "outer group/inner group/job");
/// # });
/// ```
#[derive(Debug)]
pub struct JobId {
	/// Name of the job
	pub job_name: Option<StaticStr>,

	/// List of all job group names this job belongs to.
	/// Starts at the inner job group and goes to the outer
	pub group_hierarchy: Vec<StaticStr>,
}

impl<J> JobGroup for Option<J>
where
	J: JobGroup,
{
	fn run(self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend
	where
		Self: Sized + 'static,
	{
		let Some(group) = self else {
			return FutureEither::Left(stream::empty());
		};

		FutureEither::Right(group.run())
	}
}

impl JobGroup for () {
	fn run(self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend
	where
		Self: Sized + 'static,
	{
		stream::empty()
	}
}

impl<G> JobGroup for (G,)
where
	G: OpaqueJob,
{
	fn run(mut self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend
	where
		Self: Sized + 'static,
	{
		stream::once(async move {
			let name = self.0.name().map(|n| StaticStr::from(n.to_owned()));
			(JobId::new(name), self.0.run().await)
		})
	}
}

macro_rules! impl_jobgroup_for_tuples {
	($($type_name:ident)+) => {
		impl<$($type_name),+> JobGroup for ($($type_name),+)
		where
			$($type_name: OpaqueJob),+
		{
			#[expect(
				non_snake_case,
				reason = "it's fine to re-use the names to make calling the macro easier"
			)]
			fn run(self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend
			where
				Self: Sized + 'static,
			{
				let (tx, rx) = mpsc::channel(128);
				let ($($type_name),+) = self;

				$(
					{
						let tx = tx.clone();
						let mut job = $type_name;
						// TODO: what if the task panics?
						spawn(async move {
							let result = OpaqueJob::run(&mut job).await;
							let id = JobId::new(job.name().map(|s| StaticStr::from(s.to_owned())));
							if let Err(e) = tx.send((id, result)).await {
								tracing::debug!(
									"run_in_parallel Stream channel closed before job result has been sent. JobId: {}",
									e.0.0
								);
							}
						});
					}
				)+

				ReceiverStream::new(rx)
			}
		}
	};
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

impl JobId {
	/// Creates a new [`JobId`] with the provided [`JobId::job_name`]
	pub fn new(job_name: Option<StaticStr>) -> Self {
		Self {
			group_hierarchy: Vec::new(),
			job_name,
		}
	}
}

impl Display for JobId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		const UNKNOWN_JOB: StaticStr = StaticStr::from_static_str("<UNKNOWN>");

		let path = self
			.group_hierarchy
			.iter()
			.rev()
			.chain(iter::once(self.job_name.as_ref().unwrap_or(&UNKNOWN_JOB)))
			.join("/");

		f.write_str(&path)
	}
}

pub(crate) fn spawn<F, T>(fut: F)
where
	F: Future<Output = T> + MaybeSend + 'static,
	T: MaybeSend + 'static,
{
	#[cfg(feature = "send")]
	tokio::spawn(fut);
	#[cfg(not(feature = "send"))]
	tokio::task::spawn_local(fut);
}
