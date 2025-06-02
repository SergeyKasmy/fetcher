/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all [`Actions`](`Action`) that a list of [`Entry`]'s can be run through to view/modify/filter it out

pub mod filters;
pub mod transforms;

use std::convert::Infallible;

use either::Either;
use transforms::async_fn::AsyncFnTransform;

use self::filters::{Filter, FilterAction};
use self::transforms::Transform;
use self::transforms::TransformAction;
use self::transforms::async_fn::IntoTransformedEntries;
use self::transforms::field::{Field, TransformField, TransformFieldAdapter};

use crate::actres_try;
use crate::cancellation_token::CancellationToken;
use crate::maybe_send::{MaybeSend, MaybeSendSync};
use crate::sinks::{Sink, SinkWrapper};
use crate::{
	entry::Entry, error::FetcherError, external_save::ExternalSave, sources::Source,
	task::entry_to_msg_map::EntryToMsgMap,
};

/// An action that modifies the list of entries in some way.
///
/// This is the most generic trait in the family of action traits.
/// Other action-like traits that exist are [`Transform`], [`TransformField`], [`Filter`], [`Sink`].
/// Every type implementing these traits could also be implemented via the main [`Action`] trait
/// but these traits provide a more focused and easy-to-use API tailored for each of their tasks.
/// Thus they are the more prefered option when implementing actions that fit into their intented use-cases.
/// All of them are adaptible into actions via helper functions [`transform`], [`transform_field`], [`filter`], and [`sink`].
///
/// For example, [`Sink`] is meant for "sinks", a place messages can be sent to, and thus [`Sink`]
/// doesn't bother itself with the internals of the [`Action`] trait.
/// Instead of the `Vec<Entry>` that all actions process and should then forward for other actions,
/// sinks just receive a [`&Message`](`crate::sinks::message::Message`) and that's all.
///
/// In other words, prefer to implement [`Transform`], [`TransformField`], [`Filter`], or [`Sink`]
/// instead of implementing [`Action`] directly,
/// and implement [`Action`] only when these traits don't fit your usecase.
// TODO: add similar explanations to all these action-like traits
pub trait Action: MaybeSendSync {
	/// The associated error type that can be returned while applying the action
	type Err: Into<FetcherError>;

	/// Apllies the action to the list of `entries` and returns them back.
	///
	/// `context` contains some parts of the [`Task`](`crate::task::Task`) that might be useful.
	fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> impl Future<Output = ActionResult<Self::Err>> + MaybeSend
	where
		S: Source,
		E: ExternalSave;
}

// TODO: make this TaskResult to allow jobs to be terminated by actions.
// Right now tasks return just a normal result with no way to stop the job without returning an error
/// Result of a call to [`Action::apply`]
#[derive(Debug)]
pub enum ActionResult<E, T = Vec<Entry>> {
	/// Action finished successfully
	///
	/// `Ok` is always expected to be `Vec<Entry>`.
	/// The only reasul it's a generic is to support conversions from the regular [`Result`] type.
	Ok(T),

	/// Action encountered an error
	Err(E),

	/// Action has been terminated and no other actions in the pipeline should be run
	Terminated,
}

/// Context provided to [`Action`]s with some useful parts of the parent [`Task`][Task].
///
/// The task itself can't be passed as `&mut Task` because then the action would be able to get a second mut reference to itself.
/// This works around that as a way to access some useful parts of the parent [`Task`][Task] without the aliasing issue.
///
/// [Task]: crate::task::Task
#[derive(Debug)]
pub struct ActionContext<'a, S, E> {
	/// The [`Task::source`](`crate::task::Task::source`) of the parent task, if any.
	pub source: Option<&'a mut S>,

	/// The [`Task::entry_to_msg_map`](`crate::task::Task::entry_to_msg_map`) of the parent task, if any.
	pub entry_to_msg_map: Option<&'a mut EntryToMsgMap<E>>,

	/// The [`Task::tag`](`crate::task::Task::tag`) of the parent task, if any.
	pub tag: Option<&'a str>,

	/// The [`Job::cancel_token`](`crate::job::Job::cancel_token`) of the parent job, if any.
	pub cancel_token: Option<&'a CancellationToken>,
}

/// Transforms the provided [`Filter`] into an [`Action`]
pub fn filter<F>(f: F) -> FilterAction<F>
where
	F: Filter,
{
	FilterAction(f)
}

/// Transforms the provided [`Transform`] into an [`Action`]
pub fn transform<T>(t: T) -> TransformAction<T>
where
	T: Transform,
{
	TransformAction(t)
}

/// Transforms the provided [`TransformField`] into an [`Action`] action on `field`
pub fn transform_field<T>(field: Field, t: T) -> TransformAction<TransformFieldAdapter<T>>
where
	T: TransformField,
{
	transform(TransformFieldAdapter {
		field,
		transformator: t,
	})
}

/// Transforms the provided [`TransformField`] into an [`Action`] action on [`Message::Body`](`crate::sinks::Message::body`)
pub fn transform_body<T>(t: T) -> TransformAction<TransformFieldAdapter<T>>
where
	T: TransformField,
{
	transform_field(Field::Body, t)
}

/// Transforms the provided async function implementing [`Transform`] into an [`Action`] action.
pub fn transform_fn<F, Fut, T>(f: F) -> TransformAction<AsyncFnTransform<F>>
where
	F: Fn(Entry) -> Fut + MaybeSendSync,
	Fut: Future<Output = T> + MaybeSend,
	T: IntoTransformedEntries,
{
	transform(transforms::async_fn::AsyncFnTransform(f))
}

/// Transforms the provided [`Sink`] into an [`Action`]
pub fn sink<S>(s: S) -> impl Action
where
	S: Sink,
{
	SinkWrapper(s)
}

// "&mut ActionContext" is not Copy.
// This macro allows to pass a "copy" of the context to a function
// and still be able to use the context when the function exits.
macro_rules! reborrow_ctx {
	($ctx:expr) => {{
		let ctx = $ctx;
		ActionContext {
			source: ctx.source.as_deref_mut(),
			entry_to_msg_map: ctx.entry_to_msg_map.as_deref_mut(),
			tag: ctx.tag.as_deref(),
			cancel_token: ctx.cancel_token.as_deref(),
		}
	}};
}

impl Action for () {
	type Err = Infallible;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		_context: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err>
	where
		S: Source,
		E: ExternalSave,
	{
		ActionResult::Ok(entries)
	}
}

impl<A> Action for Option<A>
where
	A: Action,
{
	type Err = A::Err;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err>
	where
		S: Source,
		E: ExternalSave,
	{
		let Some(act) = self else {
			// do nothing, just passthrough
			return ActionResult::Ok(entries);
		};

		act.apply(entries, context).await
	}
}

impl<A1, A2> Action for Either<A1, A2>
where
	A1: Action,
	A2: Action,
{
	type Err = FetcherError;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err>
	where
		S: Source,
		E: ExternalSave,
	{
		match self {
			Either::Left(x) => x.apply(entries, context).await.map_err(Into::into),
			Either::Right(x) => x.apply(entries, context).await.map_err(Into::into),
		}
	}
}

impl Action for Infallible {
	type Err = Infallible;

	async fn apply<S, E>(
		&mut self,
		_entries: Vec<Entry>,
		_context: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err>
	where
		S: Source,
		E: ExternalSave,
	{
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl Action for ! {
	type Err = !;

	async fn apply<S, E>(
		&mut self,
		_entries: Vec<Entry>,
		_context: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err>
	where
		S: Source,
		E: ExternalSave,
	{
		match *self {}
	}
}

impl<A> Action for &mut A
where
	A: Action,
{
	type Err = A::Err;

	fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> impl Future<Output = ActionResult<Self::Err>> + MaybeSend
	where
		S: Source,
		E: ExternalSave,
	{
		(*self).apply(entries, context)
	}
}

impl<A> Action for (A,)
where
	A: Action,
{
	type Err = A::Err;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err>
	where
		S: Source,
		E: ExternalSave,
	{
		self.0.apply(entries, context).await
	}
}

macro_rules! impl_action_for_tuples {
	($($type_name:ident)+) => {
		impl<$($type_name),+> Action for ($($type_name),+)
		where
			$($type_name: Action),+
		{
			type Err = FetcherError;

			#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
			async fn apply<S, E>(
				&mut self,
				entries: Vec<Entry>,
				mut ctx: ActionContext<'_, S, E>,
			) -> ActionResult<Self::Err>
			where
				S: Source,
				E: ExternalSave,
			{
				// following code expands into something like this
				//let entries = self
				//	.0
				//	.apply(entries, reborrow_ctx!(&mut ctx))
				//	.await
				//	.map_err(Into::into)?;
				//let entries = self.1.apply(entries, ctx).await.map_err(Into::into)?;

				let mut action_num = 0;
				let ($($type_name),+) = self;
				$(
					if ctx.cancel_token.as_ref().is_some_and(|tok| tok.is_cancelled()) {
						// TODO: is this fine? Maybe it shouldn't stop if a previous action had sideeffects?
						tracing::debug!("Task terminated while in the middle of action pipeline execution. Not all have actions have been run to completion.");
						return ActionResult::Terminated;
					}

					#[allow(unused_assignments, reason = "last iteration won't use it, it's fine")]
					{
						tracing::trace!("Running action #{action_num}");
						action_num += 1;
					}

					let act_result = $type_name.apply(entries, reborrow_ctx!(&mut ctx)).await;
					let entries = actres_try!(act_result.map_err(Into::into));
				)+

				ActionResult::Ok(entries)
			}
		}
	}
}

impl_action_for_tuples!(A1 A2);
impl_action_for_tuples!(A1 A2 A3);
impl_action_for_tuples!(A1 A2 A3 A4);
impl_action_for_tuples!(A1 A2 A3 A4 A5);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12);

impl<E> ActionResult<E> {
	/// Maps an `ActionResult<E>` to `ActionResult<F>` by applying a function to the contained `Err` value
	pub fn map_err<O, F>(self, op: O) -> ActionResult<F>
	where
		O: FnOnce(E) -> F,
	{
		match self {
			ActionResult::Ok(items) => ActionResult::Ok(items),
			ActionResult::Err(e) => ActionResult::Err(op(e)),
			ActionResult::Terminated => ActionResult::Terminated,
		}
	}
}

impl<T, E> From<Result<T, E>> for ActionResult<E, T> {
	fn from(value: Result<T, E>) -> Self {
		match value {
			Ok(t) => ActionResult::Ok(t),
			Err(e) => ActionResult::Err(e),
		}
	}
}

/// Unwraps an [`ActionResult`] or propagates its error or terminated branches.
///
/// Analagous to the old [`std::try`] macro which got replaced with the `?` operator.
///
/// This macro applies [`ActionResult::from`] to the passed-in value
/// which makes it possible to pass regular results to it to propagate the error as an [`ActionResult::Err`]
///
/// # Examples
///
/// ```
/// use fetcher::{
///     actres_try,
///     entry::Entry,
///     actions::ActionResult,
/// };
///
/// fn action_result() -> ActionResult<i32> {
///     let ok: ActionResult<i32> = ActionResult::Ok(vec![Entry::default()]);
///     assert_eq!(actres_try!(ok), vec![Entry::default()]);  // unwraps and returns `vec![Entry::default()]`
///
///     let terminated: ActionResult<i32> = ActionResult::Terminated;  // works the same with an `ActionResult::Err`
///     actres_try!(terminated);  // returns from the function with `ActionResult::Terminated`
///
///     unreachable!();
/// }
///
/// fn regular_result() -> ActionResult<i32> {
///     let ok: Result<&str, i32> = Ok("hello");
///     assert_eq!(actres_try!(ok), "hello"); // unwraps and returns "hello"
///
///     let err: Result<(), i32> = Err(13);
///     actres_try!(err);  // returns from the function with `ActionResult::Err(13)`
///
///     unreachable!();
/// }
/// ```
#[macro_export]
macro_rules! actres_try {
	($res:expr $(,)?) => {
		match ActionResult::from($res) {
			ActionResult::Ok(items) => items,
			ActionResult::Err(e) => return ActionResult::Err(From::from(e)),
			ActionResult::Terminated => return ActionResult::Terminated,
		}
	};
}

impl Default for ActionContext<'_, (), ()> {
	fn default() -> Self {
		Self {
			source: None,
			entry_to_msg_map: None,
			tag: None,
			cancel_token: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use std::time::{Duration, Instant};

	use tokio::join;

	use crate::{Task, actions::transform_fn, cancellation_token::CancellationToken};

	#[tokio::test]
	async fn cancel_token_stops_task_mid_work() {
		const ACTION_DELAY_MS: u64 = 200;

		let (cancel_token, tx) = CancellationToken::new();

		let request_stop_in_100ms = async move {
			tokio::time::sleep(Duration::from_millis(100)).await;
			tx.send(()).unwrap();
		};

		let long_noop_transform = async |entry| {
			tokio::time::sleep(Duration::from_millis(ACTION_DELAY_MS)).await;
			entry
		};

		let pipeline = (
			transform_fn(long_noop_transform),
			transform_fn(long_noop_transform),
			transform_fn(long_noop_transform),
		);

		let mut task = Task::<(), _, _>::builder("test")
			.action(pipeline)
			.cancel_token(cancel_token)
			.build_without_replies();

		let now = Instant::now();

		let (task_res, ()) = join!(task.run(), request_stop_in_100ms);
		task_res.unwrap();

		let elapsed = now.elapsed();
		let delay_of_3_actions = Duration::from_millis(
			ACTION_DELAY_MS * 3, /* number of actions in the pipeline */
		);

		assert!(
			elapsed < delay_of_3_actions,
			"{}ms should be less than {} * 3 = {}",
			elapsed.as_millis(),
			ACTION_DELAY_MS,
			ACTION_DELAY_MS * 3,
		);
	}
}
