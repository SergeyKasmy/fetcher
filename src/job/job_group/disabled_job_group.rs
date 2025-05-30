/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`DisabledJobGroup`] type.

use futures::{Stream, stream};

use crate::{job::JobResult, maybe_send::MaybeSend};

use super::{JobGroup, JobId};

/// Wraps a [`JobGroup`] implementation but doesn't do anything when asked to run.
///
/// See [`JobGroup::disable`].
pub struct DisabledJobGroup<G>(pub G);

impl<G> DisabledJobGroup<G> {
	/// Gets the wrapped job group out of [`DisabledJobGroup`].
	///
	/// Pattern matching can be used as well.
	pub fn into_inner(self) -> G {
		self.0
	}
}

impl<G> JobGroup for DisabledJobGroup<G>
where
	G: JobGroup,
{
	fn run_concurrently(&mut self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend {
		stream::empty()
	}

	#[cfg(feature = "send")]
	fn run_in_parallel(self) -> impl Stream<Item = (JobId, JobResult)> + Send
	where
		Self: Sized + 'static,
	{
		stream::empty()
	}
}
