mod run_result;

pub use run_result::RunResult;
use tokio::join;

use crate::{error::FetcherError, task::OpaqueTask};

pub trait TaskGroup {
	type RunResult: RunResult;

	async fn run_concurrently(&mut self) -> Self::RunResult;
}

impl<T> TaskGroup for T
where
	T: OpaqueTask,
{
	type RunResult = std::iter::Once<Result<(), FetcherError>>;

	async fn run_concurrently(&mut self) -> Self::RunResult {
		std::iter::once(OpaqueTask::run(self).await)
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
}

macro_rules! impl_taskgroup_for_tuples {
	{
		size = $size:expr;
		type_names = $($type_name:ident)+;
		indices = $($index:tt)+
	} => {
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
				results.into()
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
