/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This example showcases a job that runs every 30 minutes and
//! sends all emails in our Gmail mailbox that contain a "new release" notification from Github
//! to a Telegram group.
//!
//! In the process, the emails are moved to the archive
//! and their footer gets trimmed to make the message pretier.

use std::{env, error::Error, time::Duration};

use fetcher::{
	Job, Task,
	actions::{sink, transform_body, transforms::field::Replace},
	auth,
	job::{JobResult, RefreshTime},
	scaffold::{InitResult, init},
	sinks::Telegram,
	sources::{
		Email,
		email::{self, ViewMode},
	},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
	// Initialize the default logging framework and a detached thread waiting for a Ctrl-C signal
	//
	// The Ctrl-C signal channel enables jobs to finish more gracefully.
	// Instead of just exiting outright, the job can stop between actions
	// when the last action has already run to completion
	// or when the job is paused (be it because it's not its time yet to re-run
	// or because e.g. [`ExponentialBackoff`] error handler paused the job to wait out the error).
	let InitResult {
		ctrl_c_signal_channel,
	} = init();

	// Create a new instance of an OAuth2 authenticator for Google
	let auth = auth::Google::new(
		env::var("GOOGLE_CLIENT_ID")?,
		env::var("GOOGLE_CLIENT_SECRET")?,
		env::var("GOOGLE_REFRESH_TOKEN")?,
	);

	// Look for emails from "notifications@github.com" where the subject contains the word "release"
	let filters = email::Filters::builder()
		.sender("notifications@github.com")
		.subject("release")
		.build();

	// Set up a new email source.
	// This will be used to fetch our entries (in this case, emails) and mark the emails as read after they are sent to the sink.
	let email_source = Email::new_gmail()
		// our email address
		.email("example@gmail.com")
		// use the newly created Google OAuth2 authenticator with our credentials
		.auth(auth)
		// look for release emails by github
		.filters(filters)
		// delete these emails after they are sent (move to the archive in the usual case for Gmail)
		.view_mode(ViewMode::Delete)
		.call();

	// Create actions. Each entry will be run through each of these actions one by one
	let actions = (
		// "Transform" (modify) the body of the message.
		// Look for the usual github footer and everything after it and replace it with "" (empty string - nothing)
		transform_body(Replace::new(
			"(?s)(You are receiving this because you are subscribed to this thread).*",
			"",
		)?),
		// Send these entries to a telegram chat with chat ID -123456789 from a telegram bot using the provided bot token
		sink(Telegram::new(env::var("TELEGRAM_BOT_TOKEN")?, -123456789)),
	);

	// Create the actual task (named "github releases") - the thing that actually does the work/runs the pipeline.
	// Tasks do their work once and return immediately after.
	let task = Task::builder("github releases")
		.source(email_source)
		.action(actions)
		.build_without_replies();

	// Create a job (also named "github releases") that will contain our tasks.
	// Jobs handle re-running the tasks they contain, stopping when signaled to, and handling errors that occur during the execution of its children tasks.
	let mut job = Job::builder("github releases")
		.tasks(task)
		// the task will re-run every 30 minutes
		.refresh_time(RefreshTime::Every(Duration::from_secs(
			30 /* m */ * 60, /* secs in a min */
		)))
		// the job and the task will be stopped mid-work when they receive a signal
		.ctrlc_chan(Some(ctrl_c_signal_channel))
		// Use the default error handling that uses exponential backoff when an error occures while the tasks executes.
		// This means that the job will stop, go pause (go to sleep), and re-run the task once more a bit later.
		// If an error still occures, it will repeat this process but with a longer pause.
		// This will continue until too much pauses have been made (see [`DEFAULT_MAX_RETRY_COUNT`](`fetcher::job::error_handling::ExponentialBackoff::DEFAULT_MAX_RETRY_COUNT`)).
		// Afterwards the job will just stop forever and return its last error.
		.build_with_default_error_handling();

	// Start the job.
	// The job will run forever unless too many errors occured or Ctrl-C has been pressed
	let result = job.run().await;

	match result {
		// The job finished successfully. In our case can only happen when Ctrl-C has been pressed.
		JobResult::Ok => Ok(()),
		// The job finished with an error. This means the task have failed somwhere in the pipeline (e.g. the source or the actions).
		// JobResult::Err contains a vector for results of each contained tasks which in our case is just the one we have.
		JobResult::Err(mut errors) => Err(Box::new(errors.remove(0)) as Box<_>),
		// The job panicked. This probably shouldn't happen...
		JobResult::Panicked { payload: _ } => Ok(()),
	}
}
