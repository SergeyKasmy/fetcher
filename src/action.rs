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
use transforms::field::{Field, TransformField, TransformFieldWrapper};

use self::filters::{Filter, FilterWrapper};
use self::transforms::Transform;
use self::transforms::TransformWrapper;

use crate::maybe_send::{MaybeSend, MaybeSendSync};
use crate::sinks::{Sink, SinkWrapper};
use crate::{
	entry::Entry, error::FetcherError, external_save::ExternalSave, sources::Source,
	task::entry_to_msg_map::EntryToMsgMap,
};

pub trait Action: MaybeSendSync {
	type Error: Into<FetcherError>;

	fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> impl Future<Output = Result<Vec<Entry>, Self::Error>> + MaybeSend
	where
		S: Source,
		E: ExternalSave;
}

pub struct ActionContext<'a, S, E> {
	pub source: Option<&'a mut S>,
	pub entry_to_msg_map: Option<&'a mut EntryToMsgMap<E>>,
	pub tag: Option<&'a str>,
}

pub fn filter<F>(f: F) -> impl Action
where
	F: Filter,
{
	FilterWrapper(f)
}

pub fn transform<T>(t: T) -> impl Action
where
	T: Transform,
{
	TransformWrapper(t)
}

pub fn transform_field<T>(field: Field, t: T) -> impl Action
where
	T: TransformField,
{
	transform(TransformFieldWrapper {
		field,
		transformator: t,
	})
}

pub fn transform_body<T>(t: T) -> impl Action
where
	T: TransformField,
{
	transform_field(Field::Body, t)
}

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
		}
	}};
}

impl Action for () {
	type Error = Infallible;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		_context: ActionContext<'_, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		Ok(entries)
	}
}

impl<A> Action for (A,)
where
	A: Action,
{
	type Error = A::Error;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
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
			type Error = FetcherError;

			async fn apply<S, E>(
				&mut self,
				entries: Vec<Entry>,
				mut ctx: ActionContext<'_, S, E>,
			) -> Result<Vec<Entry>, Self::Error>
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
				//Ok(entries)

				#[expect(non_snake_case, reason = "it's fine to re-use the names to make calling the macro easier")]
				let ($($type_name),+) = self;
				$(let entries = $type_name.apply(entries, reborrow_ctx!(&mut ctx)).await.map_err(Into::into)?;)+

				Ok(entries)
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
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14 A15);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14 A15 A16);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14 A15 A16 A17);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14 A15 A16 A17 A18);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14 A15 A16 A17 A18 A19);
impl_action_for_tuples!(A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14 A15 A16 A17 A18 A19 A20);

impl<A> Action for Option<A>
where
	A: Action,
{
	type Error = A::Error;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		let Some(act) = self else {
			// do nothing, just passthrough
			return Ok(entries);
		};

		act.apply(entries, context).await
	}
}

impl<A1, A2> Action for Either<A1, A2>
where
	A1: Action,
	A2: Action,
{
	type Error = FetcherError;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'_, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
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
