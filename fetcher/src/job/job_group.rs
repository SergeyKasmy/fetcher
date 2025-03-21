mod combined_job_group;

use tokio::join;

use self::combined_job_group::CombinedJobGroup;
use super::OpaqueJob;
use crate::{ctrl_c_signal::CtrlCSignalChannel, error::FetcherError};

pub type JobRunResult = Result<(), Vec<FetcherError>>;

pub trait JobGroup {
	#[must_use = "this vec of results could contain errors"]
	async fn run_concurrently(&mut self) -> Vec<JobRunResult>;

	#[must_use = "this vec of results could contain errors"]
	async fn run_concurrently_interruptible(
		&mut self,
		ctrl_c_signal_channel: CtrlCSignalChannel,
	) -> Vec<JobRunResult>;

	fn and<G>(self, other: G) -> CombinedJobGroup<Self, G>
	where
		Self: Sized,
		G: JobGroup,
	{
		CombinedJobGroup(self, other)
	}
}

impl<J> JobGroup for J
where
	J: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		vec![OpaqueJob::run(self).await]
	}

	async fn run_concurrently_interruptible(
		&mut self,
		ctrl_c_signal_channel: CtrlCSignalChannel,
	) -> Vec<JobRunResult> {
		vec![OpaqueJob::run_interruptible(self, ctrl_c_signal_channel).await]
	}
}

impl<J1> JobGroup for (J1,)
where
	J1: OpaqueJob,
{
	async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
		vec![OpaqueJob::run(&mut self.0).await]
	}

	async fn run_concurrently_interruptible(
		&mut self,
		ctrl_c_signal_channel: CtrlCSignalChannel,
	) -> Vec<JobRunResult> {
		vec![OpaqueJob::run_interruptible(&mut self.0, ctrl_c_signal_channel).await]
	}
}

macro_rules! impl_jobgroup_for_tuples {
	($($type_name:ident)+) => {
		impl<$($type_name),+> JobGroup for ($($type_name),+)
		where
			$($type_name: OpaqueJob),+
		{
			async fn run_concurrently(&mut self) -> Vec<JobRunResult> {
				// followind expand code does something like this
				//let results = join!(self.0.run(), self.1.run());
				//vec![results.0, results.1]

				// first $type_name = specific job
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = self;

				// now $type_name = job run result
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = join!($($type_name.run()),+);
				vec![$($type_name),+]
			}

			async fn run_concurrently_interruptible(&mut self, ctrl_c_signal_channel: CtrlCSignalChannel) -> Vec<JobRunResult> {
				// first $type_name = specific job
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = self;

				// now $type_name = job run result
				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = join!($($type_name.run_interruptible(ctrl_c_signal_channel.clone())),+);
				vec![$($type_name),+]
			}
		}
	}
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
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16 J17);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16 J17 J18);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16 J17 J18 J19);
// impl_jobgroup_for_tuples!(J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 J13 J14 J15 J16 J17 J18 J19 J20);
