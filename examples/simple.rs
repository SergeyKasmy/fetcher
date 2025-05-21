//! This example showcases the simplest recurring job there can be in fetcher.
//!
//! It showcases how to create a [`Task`] that fetches the contents of <http://example.com>, properly parses its HTML to extract a title and a body, and sends it to stdout.
//! It also showcases how to create a [`Job`] that reruns this task every second and ignores all errors.

/*
// Move this to a different example

// scaffold::init() provides initializes a default logging framework (tracing), as well a Ctrl-C handling channel.
// This is useful for small applications as a starting point and can be replaced by a manual implementation as soon as needed.
let InitResult {
	ctrl_c_signal_channel,
} = scaffold::init();

// Create a read filter that will keep track which entries have already been read and which have not
let read_filter = Arc::new(RwLock::new(read_filter::Newer::new()));
*/

use std::{error::Error, time::Duration};

use fetcher::{
	actions::{
		sink, transform,
		transforms::{Html, html::DataLocation},
	},
	job::{Job, TimePoint, error_handling},
	sinks::Stdout,
	sources::{Http, SourceWithSharedRF},
	task::Task,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
	// TODO: make SourceWithSharedRF more user-friendly
	// Create a new source that fetches data from example.com and doesn't keep track if it has read it or not
	let source = SourceWithSharedRF {
		source: Http::new_get("http://example.com")?,
		rf: (),
	};

	// Create a tuple that contains all actions and executes them one by one in order
	let actions = (
		// Define a new transform that sets the title of the message to <h1> and the body to <p> from the HTML. Uses CSS selectors
		transform(
			Html::builder()
				.title("h1", Some(DataLocation::Text))?
				.text("p", Some(DataLocation::Text))?
				.build(),
		),
		// Define a sink that just prints all messages to stdout
		sink(Stdout),
	);

	// Create a new task named "example" that fetches data from the source and executes the actions on the data one by one in order
	let task = Task::builder("example")
		.source(source)
		.action(actions)
		.build_without_replies();

	// Create a new job that reruns the task every seconds and ignores all errors
	let mut job = Job::builder("example job")
		.tasks(task)
		.refresh_time(Some(TimePoint::Duration(Duration::from_secs(1))))
		.ctrlc_chan(None)
		.error_handling(error_handling::LogAndIgnore)
		.build();

	// Run the job.
	// Since the job just logs and doesn't return any errors, this will run forever.
	job.run().await.expect("never errors");

	Ok(())
}
