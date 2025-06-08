/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Filter`] trait that can be implemented in filters as well as all types that implement it

pub mod contains;
pub mod error;
pub mod take;

pub use self::{contains::Contains, take::Take};

use self::error::FilterError;
use crate::{
	entry::Entry,
	maybe_send::{MaybeSend, MaybeSendSync},
};

use std::{convert::Infallible, ops::RangeBounds, slice, vec};

use super::{Action, ActionContext, ActionResult};

/// An adapter of [`Action`] tailored for filtering out entries, i.e. deciding which entries should be keept and which should not.
pub trait Filter: MaybeSendSync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<FilterError>;

	/// Filters the vector of entries
	///
	/// # Errors
	/// Refer to implementator's docs.
	fn filter(
		&mut self,
		entries: FilterableEntries<'_>,
	) -> impl Future<Output = Result<(), Self::Err>> + MaybeSend;
}

/// Wrapper around a `&mut Vec<Entry>` that prevents modifying the contained entries themselves.
///
/// Only methods that modify the [`Vec`] itself are provided as a way to guard implementors against accidentally modifying contained entries.
#[derive(Debug)]
pub struct FilterableEntries<'a>(&'a mut Vec<Entry>);

impl<'a> FilterableEntries<'a> {
	/// Creates a new [`FilterableEntries`]
	///
	/// that allows filtering entries from the provided `Vec<Entry>` while disallowing modifying the entries themselves.
	pub fn new(entries: &'a mut Vec<Entry>) -> Self {
		Self(entries)
	}

	/// Returns the number of elements in the vector.
	///
	/// See [`Vec::len`].
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Returns an iterator over shared references to [`Entry`].
	///
	/// See [`[Entry]::iter`].
	pub fn iter(&self) -> slice::Iter<'_, Entry> {
		self.0.iter()
	}

	/// Retains only the elements specified by the predicate.
	///
	/// See [`Vec::retain`].
	pub fn retain<F>(&mut self, f: F)
	where
		F: FnMut(&Entry) -> bool,
	{
		self.0.retain(f);
	}

	/// Shortens the vector, keeping the first `len` entries and dropping the rest.
	///
	/// See [`Vec::truncate`].
	pub fn truncate(&mut self, len: usize) {
		self.0.truncate(len);
	}

	/// Removes the subslice indicated by the given range from the vector,  
	/// returning a double-ended iterator over the removed subslice.
	///
	/// See [`Vec::drain`].
	pub fn drain<R>(&mut self, range: R) -> vec::Drain<'_, Entry>
	where
		R: RangeBounds<usize>,
	{
		self.0.drain(range)
	}
}

/// Adapt a [`Filter`] to implement [`Action`] by actually filtering the entries
#[derive(Clone, Debug)]
pub struct FilterAction<F>(pub F);

impl Filter for () {
	type Err = Infallible;

	async fn filter(&mut self, _entries: FilterableEntries<'_>) -> Result<(), Self::Err> {
		Ok(())
	}
}

impl<F: Filter> Filter for Option<F> {
	type Err = F::Err;

	async fn filter(&mut self, entries: FilterableEntries<'_>) -> Result<(), Self::Err> {
		let Some(f) = self else {
			return Ok(());
		};

		f.filter(entries).await
	}
}

impl Filter for Infallible {
	type Err = Infallible;

	async fn filter(&mut self, _entries: FilterableEntries<'_>) -> Result<(), Self::Err> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl Filter for ! {
	type Err = !;

	async fn filter(&mut self, _entries: FilterableEntries<'_>) -> Result<(), Self::Err> {
		match *self {}
	}
}

impl<F> Filter for &mut F
where
	F: Filter,
{
	type Err = F::Err;

	fn filter(
		&mut self,
		entries: FilterableEntries<'_>,
	) -> impl Future<Output = Result<(), Self::Err>> + MaybeSend {
		(*self).filter(entries)
	}
}

impl<F> Action for FilterAction<F>
where
	F: Filter,
{
	type Err = FilterError;

	async fn apply<S, E>(
		&mut self,
		mut entries: Vec<Entry>,
		_ctx: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err> {
		match self.0.filter(FilterableEntries(&mut entries)).await {
			Ok(()) => ActionResult::Ok(entries),
			Err(e) => ActionResult::Err(e.into()),
		}
	}
}
