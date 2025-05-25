/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`NamedJobGroup`] type

use crate::StaticStr;

use super::JobGroup;

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
	async fn run_concurrently(&mut self) -> super::JobGroupResult {
		self.inner.run_concurrently().await
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
