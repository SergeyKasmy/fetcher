// FIXME
#![allow(missing_docs)]

use std::error::Error;
use std::{convert::Infallible, time::Duration};

use fetcher::job::error_handling::Forward;
use fetcher::{
	Job, Task,
	actions::{Action, ActionContext, ActionResult, transform_fn},
	entry::Entry,
	external_save::ExternalSave,
	job::{JobGroup, RefreshTime},
	scaffold,
	sources::Source,
};
use futures::StreamExt;

#[derive(Default)]
struct RunXTimes<const TIMES: usize>(usize);

#[derive(thiserror::Error, Debug)]
#[error("finished")]
struct FinishedError;

impl<const TIMES: usize> Action for RunXTimes<TIMES> {
	type Err = Box<dyn Error + Send + Sync>;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		_context: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err>
	where
		S: Source,
		E: ExternalSave,
	{
		eprintln!("Executing RunXTimes: {}/{TIMES}", self.0);
		if self.0 < TIMES {
			self.0 += 1;
			ActionResult::Ok(entries)
		} else {
			ActionResult::Err(FinishedError.into())
		}
	}
}

#[tokio::test(flavor = "multi_thread")]
async fn main() {
	scaffold::set_up_logging().unwrap();

	let task_never_panics = Task::<(), _, _>::builder("never_panics")
		.action(RunXTimes::<2>::default())
		.build_without_replies();

	#[expect(unreachable_code)]
	let task_always_panics = Task::<(), _, _>::builder("always_panics")
		.action(transform_fn(async |_| panic!() as Infallible))
		.build_without_replies();

	let job_never_panics = Job::builder("never_panics")
		.tasks(task_never_panics)
		.refresh_time(RefreshTime::Every(Duration::from_secs(1)))
		.ctrlc_chan(None)
		.error_handling(Forward)
		.build();

	let job_always_panics = Job::builder("always_panics")
		.tasks(task_always_panics)
		.refresh_time(RefreshTime::Every(Duration::from_secs(1)))
		.ctrlc_chan(None)
		.error_handling(Forward)
		.build();

	let group_never_panics = (job_never_panics,).with_name("group never panics");
	let group_always_panics = (job_always_panics,).with_name("group always panics");

	let group = group_never_panics
		.combine_with(group_always_panics)
		.with_name("common group");

	let mut stream = group.run();

	while let Some(res) = stream.next().await {
		eprintln!("Job {} finished! {:?}", res.0, res.1);
	}
}
