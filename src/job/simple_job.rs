/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::convert::Infallible;

use staticstr::StaticStr;

use crate::{Task, actions::Action, cancellation_token::CancellationToken, sources::Source};

use super::{HandleError, Job, Trigger, error_handling::ExponentialBackoff};

#[derive(bon::Builder, Debug)]
#[builder(finish_fn(
	name = build_internal,
	vis = ""
))]
pub struct SimpleJob<S, A, Tr, H> {
	/// Name of the job
	#[builder(start_fn, into)]
	pub name: StaticStr,

	/// Optional tag that a [`Sink`](`crate::sinks::Sink`) may put near a message body to differentiate this task from others that may be similar.
	///
	/// For example, messages from different task that are sent to the same sink can be differentiated using this adjecent tag.
	#[builder(into)]
	pub tag: Option<StaticStr>,

	/// Source where to fetch the data from.
	///
	/// Also used to mark the entry as read after it's been sent.
	pub source: Option<S>,

	/// Pipeline (in other words, a list of actions) which the data received from the source is run through
	pub action: Option<A>,

	/// Trigger the job at the provided intervals or when the trigger condition is met
	pub trigger: Tr,

	/// Handler for errors that occur during job execution
	pub error_handling: H,

	/// Gracefully stop the job when signalled
	#[builder(required)]
	pub cancel_token: Option<CancellationToken>,
}

impl<S, A, Tr, H, State: simple_job_builder::State> SimpleJobBuilder<S, A, Tr, H, State>
where
	S: Source,
	A: Action,
	Tr: Trigger,
	H: HandleError<Tr>,
{
	pub fn build(self) -> Job<Task<S, A, Infallible>, Tr, H>
	where
		State: simple_job_builder::IsComplete,
	{
		let SimpleJob {
			name,
			tag,
			source,
			action,
			trigger,
			error_handling,
			cancel_token,
		} = self.build_internal();

		let task = Task::<S, A, Infallible>::builder(name.clone())
			.maybe_tag(tag)
			.maybe_source(source)
			.maybe_action(action)
			.maybe_cancel_token(cancel_token.clone())
			.build_without_replies();

		Job::builder(name)
			.tasks(task)
			.trigger(trigger)
			.error_handling(error_handling)
			.cancel_token(cancel_token)
			.build()
	}
}

impl<S, A, Tr, State: simple_job_builder::State>
	SimpleJobBuilder<S, A, Tr, ExponentialBackoff, State>
where
	S: Source,
	A: Action,
	Tr: Trigger,
{
	pub fn build_with_default_error_handling(
		self,
	) -> Job<Task<S, A, Infallible>, Tr, ExponentialBackoff>
	where
		State::ErrorHandling: simple_job_builder::IsUnset,
		State::Trigger: simple_job_builder::IsSet,
		State::CancelToken: simple_job_builder::IsSet,
	{
		self.error_handling(ExponentialBackoff::new()).build()
	}
}
