use tokio::join;

use crate::error::FetcherError;

use super::OpaqueJob;

pub type JobRunResult = Result<(), Vec<FetcherError>>;

pub trait JobGroup {
	#[must_use = "this vec of results could contain errors"]
	async fn run_concurrently(&mut self) -> Vec<JobRunResult>;
}

impl<J> JobGroup for J
where
	J: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		vec![OpaqueJob::run(self).await]
	}
}

impl<J1> JobGroup for (J1,)
where
	J1: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		vec![OpaqueJob::run(&mut self.0).await]
	}
}

impl<J1, J2> JobGroup for (J1, J2)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(self.0.run(), self.1.run());
		vec![results.0, results.1]
	}
}

impl<J1, J2, J3> JobGroup for (J1, J2, J3)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
	J3: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(self.0.run(), self.1.run(), self.2.run());
		vec![results.0, results.1, results.2]
	}
}

impl<J1, J2, J3, J4> JobGroup for (J1, J2, J3, J4)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
	J3: OpaqueJob,
	J4: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(self.0.run(), self.1.run(), self.2.run(), self.3.run());
		vec![results.0, results.1, results.2, results.3]
	}
}

impl<J1, J2, J3, J4, J5> JobGroup for (J1, J2, J3, J4, J5)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
	J3: OpaqueJob,
	J4: OpaqueJob,
	J5: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
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

impl<J1, J2, J3, J4, J5, J6> JobGroup for (J1, J2, J3, J4, J5, J6)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
	J3: OpaqueJob,
	J4: OpaqueJob,
	J5: OpaqueJob,
	J6: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run(),
			self.5.run()
		);
		vec![
			results.0, results.1, results.2, results.3, results.4, results.5,
		]
	}
}

impl<J1, J2, J3, J4, J5, J6, J7> JobGroup for (J1, J2, J3, J4, J5, J6, J7)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
	J3: OpaqueJob,
	J4: OpaqueJob,
	J5: OpaqueJob,
	J6: OpaqueJob,
	J7: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run(),
			self.5.run(),
			self.6.run()
		);
		vec![
			results.0, results.1, results.2, results.3, results.4, results.5, results.6,
		]
	}
}

impl<J1, J2, J3, J4, J5, J6, J7, J8> JobGroup for (J1, J2, J3, J4, J5, J6, J7, J8)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
	J3: OpaqueJob,
	J4: OpaqueJob,
	J5: OpaqueJob,
	J6: OpaqueJob,
	J7: OpaqueJob,
	J8: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run(),
			self.5.run(),
			self.6.run(),
			self.7.run()
		);
		vec![
			results.0, results.1, results.2, results.3, results.4, results.5, results.6, results.7,
		]
	}
}

impl<J1, J2, J3, J4, J5, J6, J7, J8, J9> JobGroup for (J1, J2, J3, J4, J5, J6, J7, J8, J9)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
	J3: OpaqueJob,
	J4: OpaqueJob,
	J5: OpaqueJob,
	J6: OpaqueJob,
	J7: OpaqueJob,
	J8: OpaqueJob,
	J9: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run(),
			self.5.run(),
			self.6.run(),
			self.7.run(),
			self.8.run()
		);
		vec![
			results.0, results.1, results.2, results.3, results.4, results.5, results.6, results.7,
			results.8,
		]
	}
}

impl<J1, J2, J3, J4, J5, J6, J7, J8, J9, J10> JobGroup for (J1, J2, J3, J4, J5, J6, J7, J8, J9, J10)
where
	J1: OpaqueJob,
	J2: OpaqueJob,
	J3: OpaqueJob,
	J4: OpaqueJob,
	J5: OpaqueJob,
	J6: OpaqueJob,
	J7: OpaqueJob,
	J8: OpaqueJob,
	J9: OpaqueJob,
	J10: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run(),
			self.5.run(),
			self.6.run(),
			self.7.run(),
			self.8.run(),
			self.9.run()
		);
		vec![
			results.0, results.1, results.2, results.3, results.4, results.5, results.6, results.7,
			results.8, results.9,
		]
	}
}
