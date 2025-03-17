use tokio::join;

use crate::{error::FetcherError, task::OpaqueTask};

pub trait TaskGroup {
	async fn run(&mut self) -> Vec<Result<(), FetcherError>>;
}

impl<T1> TaskGroup for T1
where
	T1: OpaqueTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		vec![OpaqueTask::run(self).await]
	}
}

impl<T1> TaskGroup for (T1,)
where
	T1: OpaqueTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		vec![self.0.run().await]
	}
}

impl<T1, T2> TaskGroup for (T1, T2)
where
	T1: OpaqueTask,
	T2: OpaqueTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		let results = join!(self.0.run(), self.1.run());
		vec![results.0, results.1]
	}
}

impl<T1, T2, T3> TaskGroup for (T1, T2, T3)
where
	T1: OpaqueTask,
	T2: OpaqueTask,
	T3: OpaqueTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		let results = join!(self.0.run(), self.1.run(), self.2.run());
		vec![results.0, results.1, results.2]
	}
}

impl<T1, T2, T3, T4> TaskGroup for (T1, T2, T3, T4)
where
	T1: OpaqueTask,
	T2: OpaqueTask,
	T3: OpaqueTask,
	T4: OpaqueTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		let results = join!(self.0.run(), self.1.run(), self.2.run(), self.3.run());
		vec![results.0, results.1, results.2, results.3]
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
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run()
		);
		vec![results.0, results.1, results.2, results.3, results.4]
	}
}
