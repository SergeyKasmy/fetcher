// FIXME
#![expect(missing_docs)]

use fetcher::job::{Job, JobGroup, Trigger, error_handling::Forward};
use futures::stream::StreamExt;
use tokio::task::LocalSet;

#[tokio::test(flavor = "current_thread")]
async fn main() {
	LocalSet::new()
		.run_until(async {
			// Create jobs
			let job1 = Job::builder("job1")
				.tasks(())
				.trigger(Trigger::Never)
				.error_handling(Forward)
				.ctrlc_chan(None)
				.build();
			let job2 = Job::builder("job2")
				.tasks(())
				.trigger(Trigger::Never)
				.error_handling(Forward)
				.ctrlc_chan(None)
				.build();
			// Group jobs using a tuple
			let group = (job1, job2);
			// Run jobs and get results
			let mut group_results = group.clone().run();
			while let Some(job_result) = group_results.next().await {
				println!("Job {} finished!", job_result.0);
			}
			drop(group_results);
			// Add a name to the group
			let named_group = group.with_name("my_group");
			// Temporarily disable the group
			let _disabled = named_group.disable();
		})
		.await;
}
