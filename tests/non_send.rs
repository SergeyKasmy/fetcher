//! This test checks that spawning non-send jobs works as expected when feature "send" is not enabled

#[cfg(not(feature = "send"))]
#[tokio::test(flavor = "current_thread")]
async fn non_send_jobs() {
	use std::{cell::Cell, rc::Rc};

	use fetcher::{
		Task,
		actions::transform_fn,
		job::{Job, error_handling::Forward, trigger},
	};
	use tokio::{join, task::LocalSet};

	LocalSet::new()
		.run_until(async {
			let rc = Rc::new(Cell::new(0));

			let task = Task::<(), _, _>::builder("task")
				.action(transform_fn(async |entry| {
					rc.update(|x| x + 1);
					entry
				}))
				.build_without_replies();

			let mut job1 = Job::builder("job1")
				.tasks(task.clone())
				.trigger(trigger::Never)
				.error_handling(Forward)
				.ctrlc_chan(None)
				.build();

			let mut job2 = Job::builder("job2")
				.tasks(task)
				.trigger(trigger::Never)
				.error_handling(Forward)
				.ctrlc_chan(None)
				.build();

			let res = join!(job1.run(), job2.run());

			assert_eq!(rc.get(), 2);
		})
		.await;
}
