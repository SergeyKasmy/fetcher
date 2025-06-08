/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This test confirms that finished jobs arrive in the [`JobGroup::run`] stream in the correct order as soon as they are finished

use std::error::Error;
use std::time::Instant;
use std::{convert::Infallible, time::Duration};

use assert_matches::assert_matches;
use fetcher::job::error_handling::Forward;
use fetcher::job::{JobResult, trigger};
use fetcher::{
	Job, Task,
	actions::{Action, ActionContext, ActionResult, transform_fn},
	entry::Entry,
	external_save::ExternalSave,
	job::JobGroup,
	sources::Source,
};
use futures::StreamExt;

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
		eprintln!("Executing RunXTimes: {}/{TIMES}", self.0 + 1);
		if self.0 < TIMES {
			self.0 += 1;
			ActionResult::Ok(entries)
		} else {
			ActionResult::Err(FinishedError.into())
		}
	}
}

impl<const TIMES: usize> Default for RunXTimes<TIMES> {
	fn default() -> Self {
		Self(1)
	}
}

#[tokio::test(flavor = "multi_thread")]
async fn job_group_stream() {
	let trigger_every_100ms = trigger::Every(Duration::from_millis(100));

	let task_never_panics = Task::<(), _, _>::builder("never_panics")
		.action(RunXTimes::<2>::default())
		.build_without_replies();

	#[expect(unreachable_code)]
	let task_always_panics = Task::<(), _, _>::builder("always_panics")
		.action(transform_fn(async |_| panic!() as Infallible))
		.build_without_replies();

	let job_never_panics = Job::builder("never_panics")
		.tasks(task_never_panics)
		.trigger(trigger_every_100ms)
		.cancel_token(None)
		.error_handling(Forward)
		.build();

	let job_always_panics = Job::builder("always_panics")
		.tasks(task_always_panics)
		.trigger(trigger_every_100ms)
		.cancel_token(None)
		.error_handling(Forward)
		.build();

	let group_never_panics = (job_never_panics,).with_name("group_never_panics");
	let group_always_panics = (job_always_panics,).with_name("group_always_panics");

	let group = group_never_panics
		.combine_with(group_always_panics)
		.with_name("common_group");

	let mut stream = group.run();
	let now = Instant::now();

	let mut first = true;
	let mut finished_jobs = Vec::new();
	while let Some((job_id, job_res)) = stream.next().await {
		eprintln!("Job {} finished! {:?}", job_id, job_res);

		let elapsed = now.elapsed();
		if first {
			// panicking job should return ASAP, normal one only after at least 250ms.
			// Test with a big margin here, but still not enough for the normal one to finish.
			assert!(
				elapsed < Duration::from_millis(50),
				"panicking job finished too late (in {}ms which is bigger than expected <200ms)",
				elapsed.as_millis()
			);
			first = false;
		} else {
			assert!(
				elapsed > Duration::from_millis(100),
				"normal non-panicking job finished too soon (in {}ms which is smaller than expected >250ms)",
				elapsed.as_millis()
			);
		}

		finished_jobs.push((job_id.to_string(), job_res));
	}

	let mut finished_jobs = finished_jobs.iter();
	assert_matches!(
		finished_jobs.next().map(tuple_as_ref),
		Some((
			"common_group/group_always_panics/always_panics",
			JobResult::Panicked { .. }
		))
	);
	assert_matches!(
		finished_jobs.next().map(tuple_as_ref),
		Some((
			"common_group/group_never_panics/never_panics",
			JobResult::Err(_)
		))
	);
	assert_matches!(finished_jobs.next(), None);
}

fn tuple_as_ref(tuple: &(String, JobResult)) -> (&str, &JobResult) {
	(&tuple.0, &tuple.1)
}
