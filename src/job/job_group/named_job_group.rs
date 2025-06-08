/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`NamedJobGroup`] type

use futures::Stream;

use super::{JobGroup, JobId};
use crate::{StaticStr, job::JobResult, maybe_send::MaybeSend};

/// A [`JobGroup`] wrapper that appends the provided name to [`JobId::group_hierarchy`] and creates a tracing span containing the name.
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
	fn run(self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend
	where
		Self: Sized + 'static,
	{
		use futures::StreamExt;

		self.inner.run().map(move |(mut job_id, job_result)| {
			job_id.group_hierarchy.push(self.name.clone());

			(job_id, job_result)
		})
	}
}
