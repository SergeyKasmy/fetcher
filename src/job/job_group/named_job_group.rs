/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`NamedJobGroup`] type

use futures::Stream;

use crate::{StaticStr, job::JobResult, maybe_send::MaybeSend};

use super::{JobGroup, JobId};

/// A [`JobGroup`] wrapper that prepends the provided name to [`JobGroup::names`] calls and creates a tracing span containing the name.
///
/// See [`JobGroup::with_name`].
pub struct NamedJobGroup<G> {
	/// Wrapped job group
	pub inner: G,

	/// Name of the contained job group
	pub name: StaticStr,
}

impl<G> JobGroup for NamedJobGroup<G>
where
	G: JobGroup,
{
	#[tracing::instrument(skip(self), fields(job_group = %self.name))]
	fn run_concurrently(&mut self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend {
		self.inner.run_concurrently()
	}

	#[cfg(feature = "send")]
	#[tracing::instrument(skip(self), fields(job_group = %self.name))]
	async fn run_in_parallel(self) -> (super::JobGroupResult, Self)
	where
		Self: 'static,
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
			let mut name = name?;

			name.insert(0, '/');
			name.insert_str(0, &self.name);

			Some(name)
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		Job,
		job::{JobGroup, JobResult, OpaqueJob, RefreshTime, error_handling::Forward},
	};

	#[test]
	fn named_job_group_doesnt_add_name_to_job_with_no_name() {
		struct UnnamedJob;

		impl OpaqueJob for UnnamedJob {
			async fn run(&mut self) -> JobResult {
				JobResult::Ok
			}

			fn name(&self) -> Option<&str> {
				None
			}
		}

		let named_job = Job::builder("named_job")
			.tasks(())
			.refresh_time(RefreshTime::Never)
			.error_handling(Forward)
			.ctrlc_chan(None)
			.build();

		let unnamed_job = UnnamedJob;

		let group = (named_job, unnamed_job);
		assert_eq!(
			group.names().collect::<Vec<_>>(),
			[Some("named_job".to_owned()), None]
		);

		let named_group = group.with_name("named_group");
		assert_eq!(
			named_group.names().collect::<Vec<_>>(),
			[Some("named_group/named_job".to_owned()), None]
		);
	}
}
