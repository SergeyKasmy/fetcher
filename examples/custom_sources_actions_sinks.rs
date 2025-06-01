/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This example showcases how to create and use custom
//! [`Sources`](`Source`), [`Actions`](`Action`) ([`Transforms`](`Transform`) and [`Filters`](`Filter`)), and [`Sinks`](`Sink`).
//!
//! This example defines:
//! * a [`Source`] that generates entries with message bodies that contain the current unix epoch time ([`UnixEpochTimeSource`]).
//! * a [`TransformField`] that just logs the value of the provided field and does nothing else ([`LogFieldTransform`]).
//! * a [`Filter`] that filters every 10th entry that passes through it ([`FilterEveryTenthEntry`]).
//! * a [`Sink`] that writes the contents of message bodies passed to it to a file, every time overwriting the old value ([`SaveBodyToFileSink`]).

#![allow(clippy::dbg_macro)]

use std::{
	convert::Infallible,
	error::Error,
	time::{Duration, SystemTime, UNIX_EPOCH},
};

use fetcher::{
	Job, Task,
	actions::{
		filter,
		filters::Filter,
		sink, transform_body, transform_fn,
		transforms::{field::TransformField, result::TransformResult},
	},
	entry::Entry,
	job::{JobResult, Trigger, error_handling},
	scaffold::{InitResult, init},
	sinks::{Message, Sink, message::MessageId},
	sources::Fetch,
};
use tokio::{fs::File, io::AsyncWriteExt};

/// Define the type that we will implement [`Fetch`] on.
///
/// It will generate 1 entry each time it's run that will contain current unix epoch time in its message body.
///
/// [`Fetch`] is the biggest part of a [`Source`]: it handles actually fetching and returning entries
/// that will run through the pipeline.
///
/// [`Source`] has other responsobilities besides just fetching:
/// it has to provide a way to mark which entries have been read and which have not.
///
/// Some sources keep this state in themselves, e.g. [`Email`](`fetcher::sinks::Email`),
/// others (most of them) keep track of it via external means.
/// Types that facilitate keeping track of read and unread entries must implement [`ReadFilter`](`fetcher::read_filter::ReadFilter`).
///
/// In other words, a [`Source`] is just a [`Fetch`] and a [`ReadFilter`].
///
/// Our type doesn't need to keep track of this information, so we will just implement [`Fetch`] for it
/// and use it with a no-op [`ReadFilter`] via [`Fetch::into_source_without_read_filter`].
/// This is in contast to [`Fetch::into_source_with_read_filter`] which takes an external read filter
/// to keep track of our entry read status.
struct UnixEpochTimeSource;

impl Fetch for UnixEpochTimeSource {
	/// [`UnixEpochTimeSource`] never errors
	type Err = Infallible;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		// calculate time since unix epoch
		let unix_epoch_time = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("system time shouldn't be set to before 1970");

		// TODO: add and use builders
		// create an entry and put that time into its message body
		let entry = Entry {
			msg: Message {
				body: Some(unix_epoch_time.as_secs().to_string()),
				..Default::default()
			},
			..Default::default()
		};

		Ok(vec![entry])
	}
}

/// Define the type that we will implement [`TransformField`] on.
///
/// It will just log current message body and do nothing else.
///
/// [`TransformField`] is a specialization of [`Transform`] that works specifically on fields of an entry.
/// This is true for most string manipulation transforms.
/// A [`TransformField`] can be used on any field and thus the field we will work on is defined on usage site.
struct LogFieldTransform;

impl TransformField for LogFieldTransform {
	/// [`LogFieldTransform`] never errors
	type Err = Infallible;

	// TODO: rename old_val to something better
	/// `old_val` contains the value the field currently contains
	fn transform_field(
		&mut self,
		old_val: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		let time = old_val.expect("our source always returns entries with bodies");

		// just log it
		tracing::info!("Current time is {time}!");

		// and specify that we want to keep the previous value as is
		Ok(TransformResult::Previous)
	}
}

/// Define the type that we will implement [`Filter`] on.
///
/// It will remove every 10th entry and stop it from getting further in the pipeline.
struct FilterEveryTenthEntry(usize);

impl Filter for FilterEveryTenthEntry {
	/// [`FilterEveryTenthEntry`] never errors
	type Err = Infallible;

	async fn filter(&mut self, entries: &mut Vec<Entry>) -> Result<(), Self::Err> {
		entries.retain(|_| {
			self.0 += 1;

			// keep the entry only if it's not 10th
			self.0 != 10
		});

		Ok(())
	}
}

/// Define the type that we will implement [`Sink`] on.
///
/// It will write bodies of the messages passed to it to a file, overwriting it every time.
///
/// We keep a handle to the open file in the type.
struct SaveBodyToFileSink(tokio::fs::File);

impl Sink for SaveBodyToFileSink {
	/// Any type convertible to Box<dyn Error> works
	type Err = Box<dyn Error + Send + Sync>;

	// This function will be called for every message of every entry that will pass through the sink.
	async fn send(
		&mut self,
		message: &Message,
		_reply_to: Option<&MessageId>,
		_tag: Option<&str>,
	) -> Result<Option<MessageId>, Self::Err> {
		// extract the body
		let msg_body = message
			.body
			.as_ref()
			.expect("source should always return entries with bodies");

		// erase the file
		self.0.set_len(0).await?;

		// write the body (our unix time) to the file
		self.0.write_all(msg_body.as_bytes()).await?;

		Ok(None)
	}
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
	// Initialize the default logging framework and a detached thread waiting for a Ctrl-C signal.
	//
	// TODO: add this to Ctrl-C chan docs
	// The Ctrl-C signal channel enables jobs to finish more gracefully.
	// Instead of just exiting outright, the job can stop between actions
	// when the last action has already run to completion
	// or when the job is paused (be it because it's not its time yet to re-run
	// or because e.g. [`ExponentialBackoff`] error handler paused the job to wait out the error)
	let InitResult {
		ctrl_c_signal_channel,
	} = init();

	// create a sink that writes the contents of the message body to file "time.txt"
	let save_to_file = SaveBodyToFileSink(
		File::options()
			.create(true)
			.write(true)
			.truncate(true)
			.open("time.txt")
			.await?,
	);

	// create the list of actions our entry will run through
	let actions = (
		filter(FilterEveryTenthEntry(0)),
		// our custom [`TransformField`] action will log the message body of every entries that comes through it
		transform_body(LogFieldTransform),
		// custom transform implementation can also be created using async functions to avoid boilerplate code,
		// especially for simple actions
		transform_fn(async |entry| dbg!(entry)),
		// our custom [`Sink`] will write the message body (that contains time since unix epoch) to the file
		sink(save_to_file),
	);

	// Create the actual task (named "example") - the thing that actually does the work/runs the pipeline.
	// Tasks do their work once and return immediately after.
	let task = Task::builder("example")
		// every time the task is run, it will fetch an entry from our source
		// which will contain the current time since unix epoch in the message body
		.source(UnixEpochTimeSource.into_source_without_read_filter())
		.action(actions)
		.build_without_replies();

	// Create a job (also named "example") that will contain our tasks.
	// Jobs handle re-running the tasks they contain, stopping when signaled to, and handling errors that occur during the execution of its children tasks.
	let mut job = Job::builder("example")
		.tasks(task)
		// the task will re-run every 5 seconds
		.trigger(Trigger::Every(Duration::from_secs(5)))
		// the job and the task will be stopped mid-work when they receive a signal
		.ctrlc_chan(Some(ctrl_c_signal_channel))
		// if an error occures, stop the job immediately and return, aka "forward" the error
		.error_handling(error_handling::Forward)
		.build();

	// Start the job.
	// The job will run until an error happens and then stop immediately
	#[expect(clippy::match_same_arms)]
	match job.run().await {
		// The job finished successfully. In our case can only happen when Ctrl-C has been pressed.
		JobResult::Ok => Ok(()),
		// The job finished with an error. This means the task have failed somwhere in the pipeline (e.g. the source or the actions).
		// JobResult::Err contains a vector for results of each contained tasks which in our case is just the one we have.
		JobResult::Err(errors) => Err(Box::new(errors.into_first()) as Box<_>),
		// The job panicked. This probably shouldn't happen...
		JobResult::Panicked { payload: _ } => Ok(()),
	}
}
