/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`TaskGroup`] trait

use tokio::join;

use crate::{
	cancellation_token::CancellationToken,
	error::FetcherError,
	maybe_send::{MaybeSend, MaybeSendSync},
	task::OpaqueTask,
};

/// A group of tasks that are run together as part of a [`Job`](`crate::job::Job`).
pub trait TaskGroup: MaybeSendSync {
	/// Result of a run of the task group.
	///
	/// An iterator-like type, yielding [`Result<(), FetcherError>`]
	type RunResult: IntoIterator<Item = Result<(), FetcherError>>;

	/// Runs all tasks in the group in parallel in the same async task.
	///
	/// This method runs all jobs in the group concurrently using [`join!()`].
	fn run_concurrently(&mut self) -> impl Future<Output = Self::RunResult> + MaybeSend;

	/// Sets the [`CancellationToken`] of all the tasks in the group to `token`.
	///
	/// By default it uses [`OpaqueTask::set_cancel_token`].
	fn set_cancel_token(&mut self, token: CancellationToken);
}

impl<T> TaskGroup for T
where
	T: OpaqueTask,
{
	type RunResult = std::iter::Once<Result<(), FetcherError>>;

	async fn run_concurrently(&mut self) -> Self::RunResult {
		std::iter::once(OpaqueTask::run(self).await)
	}

	fn set_cancel_token(&mut self, channel: CancellationToken) {
		OpaqueTask::set_cancel_token(self, channel);
	}
}

impl<T> TaskGroup for (T,)
where
	T: OpaqueTask,
{
	type RunResult = std::iter::Once<Result<(), FetcherError>>;

	async fn run_concurrently(&mut self) -> Self::RunResult {
		self.0.run_concurrently().await
	}

	fn set_cancel_token(&mut self, channel: CancellationToken) {
		self.0.set_cancel_token(channel);
	}
}

macro_rules! impl_taskgroup_for_tuples {
	{
		size = $size:expr;
		type_names = $($type_name:ident)+;
		indices = $($index:tt)+
	} => {
		#[expect(non_snake_case, reason = "it's fine to re-use the names for macro use")]
		impl<$($type_name),+> TaskGroup for ($($type_name),+)
		where
			$($type_name: OpaqueTask),+
		{
			type RunResult = [Result<(), FetcherError>; $size];

			async fn run_concurrently(&mut self) -> Self::RunResult {
				// following code expands into something like this
				//let results = join!(self.0.run(), self.1.run());
				//[results.0, results.1]

				let results = join!($(self.$index.run()),+);

				tracing::trace!("All tasks have finished");

				results.into()
			}

			fn set_cancel_token(&mut self, token: CancellationToken) {
				let ($($type_name),+) = self;
				$(
					$type_name.set_cancel_token(token.clone());
				)+
			}
		}
	}
}

impl_taskgroup_for_tuples! {
	size = 2;
	type_names = T1 T2;
	indices = 0 1
}

impl_taskgroup_for_tuples! {
	size = 3;
	type_names = T1 T2 T3;
	indices = 0 1 2
}

impl_taskgroup_for_tuples! {
	size = 4;
	type_names = T1 T2 T3 T4;
	indices = 0 1 2 3
}

impl_taskgroup_for_tuples! {
	size = 5;
	type_names = T1 T2 T3 T4 T5;
	indices = 0 1 2 3 4
}

impl_taskgroup_for_tuples! {
	size = 6;
	type_names = T1 T2 T3 T4 T5 T6;
	indices = 0 1 2 3 4 5
}

impl_taskgroup_for_tuples! {
	size = 7;
	type_names = T1 T2 T3 T4 T5 T6 T7;
	indices = 0 1 2 3 4 5 6
}

impl_taskgroup_for_tuples! {
	size = 8;
	type_names = T1 T2 T3 T4 T5 T6 T7 T8;
	indices = 0 1 2 3 4 5 6 7
}

impl_taskgroup_for_tuples! {
	size = 9;
	type_names = T1 T2 T3 T4 T5 T6 T7 T8 T9;
	indices = 0 1 2 3 4 5 6 7 8
}

impl_taskgroup_for_tuples! {
	size = 10;
	type_names = T1 T2 T3 T4 T5 T6 T7 T8 T9 T10;
	indices = 0 1 2 3 4 5 6 7 8 9
}

impl_taskgroup_for_tuples! {
	size = 11;
	type_names = T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11;
	indices = 0 1 2 3 4 5 6 7 8 9 10
}

impl_taskgroup_for_tuples! {
	size = 12;
	type_names = T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12;
	indices = 0 1 2 3 4 5 6 7 8 9 10 11
}
