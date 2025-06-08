/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This example showcases the simplest recurring job there can be in fetcher.
//!
//! It shows how to create a [`Task`] that fetches the contents of <http://example.com>, properly parses its HTML to extract a title and a body, and sends it to stdout.
//! It also shows how to create a [`Job`] that reruns this task every second and ignores all errors.
//!
//! To make this more useful as an actual job, sink should be changed from stdout to something else.

use std::{error::Error, time::Duration};

use fetcher::{
	actions::{sink, transform, transforms::Html},
	job::{Job, error_handling, trigger},
	sinks::Stdout,
	sources::{Fetch, Http},
	task::Task,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
	// Create a new source that fetches data from example.com and doesn't keep track if it has read it or not
	let source = Http::new_get("http://example.com")?.into_source_without_read_filter();

	// Create a pipeline (via a tuple) that contains all actions and executes them one by one in order
	let actions = (
		// Define a new transform that sets the title of the message to <h1> and the body to <p> from the HTML. Uses CSS selectors
		transform(Html::builder().title("h1")?.text("p")?.build()),
		// Define a sink that just prints all messages to stdout
		sink(Stdout),
	);

	// TODO: use job::builder_simple but also mention that this is the same as this
	// Create a new task named "example" that fetches data from the source and executes the actions on the data one by one in order
	let task = Task::builder("example")
		.source(source)
		.action(actions)
		.build_without_replies();

	// Create a new job that reruns the task every seconds and ignores all errors
	let mut job = Job::builder("example job")
		.tasks(task)
		.trigger(trigger::Every(Duration::from_secs(1)))
		.cancel_token(None)
		.error_handling(error_handling::LogAndIgnore)
		.build();

	// Run the job.
	// Since the job just logs and doesn't return any errors, this will run forever.
	job.run().await.expect("never errors");

	Ok(())
}
