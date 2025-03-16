/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all [`Actions`](`Action`) that a list of [`Entry`]'s can be run through to view/modify/filter it out

pub mod filters;
pub mod transforms;

use self::filters::{Filter, FilterWrapper};
use self::transforms::Transform;
use self::transforms::TransformWrapper;

use crate::sink::{Sink, SinkWrapper};
use crate::{
	entry::Entry, error::FetcherError, external_save::ExternalSave, source::Source,
	task::entry_to_msg_map::EntryToMsgMap,
};

pub trait Action {
	type Error: Into<FetcherError>;

	async fn apply<'a, S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'a, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
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

pub fn sink<S>(s: S) -> impl Action
where
	S: Sink,
{
	SinkWrapper(s)
}

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

impl<A1> Action for (A1,)
where
	A1: Action,
{
	type Error = FetcherError;

	async fn apply<'a, S, E>(
		&mut self,
		entries: Vec<Entry>,
		context: ActionContext<'a, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		self.0.apply(entries, context).await.map_err(Into::into)
	}
}

impl<A1, A2> Action for (A1, A2)
where
	A1: Action,
	A2: Action,
{
	type Error = FetcherError;

	async fn apply<'a, S, E>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;
		self.1.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3> Action for (A1, A2, A3)
where
	A1: Action,
	A2: Action,
	A3: Action,
{
	type Error = FetcherError;

	async fn apply<'a, S, E>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.2.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3, A4> Action for (A1, A2, A3, A4)
where
	A1: Action,
	A2: Action,
	A3: Action,
	A4: Action,
{
	type Error = FetcherError;

	async fn apply<'a, S, E>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.2
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.3.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3, A4, A5> Action for (A1, A2, A3, A4, A5)
where
	A1: Action,
	A2: Action,
	A3: Action,
	A4: Action,
	A5: Action,
{
	type Error = FetcherError;

	async fn apply<'a, S, E>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.2
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.3
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.4.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3, A4, A5, A6> Action for (A1, A2, A3, A4, A5, A6)
where
	A1: Action,
	A2: Action,
	A3: Action,
	A4: Action,
	A5: Action,
	A6: Action,
{
	type Error = FetcherError;

	async fn apply<'a, S, E>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.2
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.3
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.4
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.5.apply(entries, ctx).await.map_err(Into::into)
	}
}

impl<A1, A2, A3, A4, A5, A6, A7> Action for (A1, A2, A3, A4, A5, A6, A7)
where
	A1: Action,
	A2: Action,
	A3: Action,
	A4: Action,
	A5: Action,
	A6: Action,
	A7: Action,
{
	type Error = FetcherError;

	async fn apply<'a, S, E>(
		&mut self,
		entries: Vec<Entry>,
		mut ctx: ActionContext<'a, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		let entries = self
			.0
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.1
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.2
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.3
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.4
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		let entries = self
			.5
			.apply(entries, reborrow_ctx!(&mut ctx))
			.await
			.map_err(Into::into)?;

		self.6.apply(entries, ctx).await.map_err(Into::into)
	}
}
