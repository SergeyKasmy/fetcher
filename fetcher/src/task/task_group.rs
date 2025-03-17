mod run_result;

pub use run_result::RunResult;
use tokio::join;

use crate::{error::FetcherError, task::OpaqueTask};

pub trait TaskGroup {
	type RunResult: RunResult;

	async fn run_concurrently(&mut self) -> Self::RunResult;
}

impl<T1> TaskGroup for T1
where
	T1: OpaqueTask,
{
	type RunResult = std::iter::Once<Result<(), FetcherError>>;

	async fn run_concurrently(&mut self) -> Self::RunResult {
		std::iter::once(OpaqueTask::run(self).await)
	}
}

impl<T1> TaskGroup for (T1,)
where
	T1: OpaqueTask,
{
	type RunResult = std::iter::Once<Result<(), FetcherError>>;

	async fn run_concurrently(&mut self) -> Self::RunResult {
		std::iter::once(OpaqueTask::run(&mut self.0).await)
	}
}

impl<T1, T2> TaskGroup for (T1, T2)
where
	T1: OpaqueTask,
	T2: OpaqueTask,
{
	type RunResult = [Result<(), FetcherError>; 2];

	async fn run_concurrently(&mut self) -> Self::RunResult {
		let results = join!(self.0.run(), self.1.run());
		[results.0, results.1]
	}
}

impl<T1, T2, T3> TaskGroup for (T1, T2, T3)
where
	T1: OpaqueTask,
	T2: OpaqueTask,
	T3: OpaqueTask,
{
	type RunResult = [Result<(), FetcherError>; 3];

	async fn run_concurrently(&mut self) -> Self::RunResult {
		let results = join!(self.0.run(), self.1.run(), self.2.run());
		[results.0, results.1, results.2]
	}
}

impl<T1, T2, T3, T4> TaskGroup for (T1, T2, T3, T4)
where
	T1: OpaqueTask,
	T2: OpaqueTask,
	T3: OpaqueTask,
	T4: OpaqueTask,
{
	type RunResult = [Result<(), FetcherError>; 4];

	async fn run_concurrently(&mut self) -> Self::RunResult {
		let results = join!(self.0.run(), self.1.run(), self.2.run(), self.3.run());
		[results.0, results.1, results.2, results.3]
	}
}

impl<T1, T2, T3, T4, T5> TaskGroup for (T1, T2, T3, T4, T5)
where
	T1: OpaqueTask,
	T2: OpaqueTask,
	T3: OpaqueTask,
	T4: OpaqueTask,
	T5: OpaqueTask,
{
	type RunResult = [Result<(), FetcherError>; 5];

	async fn run_concurrently(&mut self) -> Self::RunResult {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run()
		);
		[results.0, results.1, results.2, results.3, results.4]
	}
}

impl<T1, T2, T3, T4, T5, T6> TaskGroup for (T1, T2, T3, T4, T5, T6)
where
	T1: OpaqueTask,
	T2: OpaqueTask,
	T3: OpaqueTask,
	T4: OpaqueTask,
	T5: OpaqueTask,
	T6: OpaqueTask,
{
	type RunResult = [Result<(), FetcherError>; 6];

	async fn run_concurrently(&mut self) -> Self::RunResult {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run(),
			self.5.run()
		);

		[
			results.0, results.1, results.2, results.3, results.4, results.5,
		]
	}
}
