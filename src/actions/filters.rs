/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Filter`] trait that can be implemented in filters as well as all types that implement it

pub mod contains;
pub mod error;
pub mod take;

use error::FilterError;

pub use self::{contains::Contains, take::Take};

use crate::{
	entry::Entry,
	maybe_send::{MaybeSend, MaybeSendSync},
};

use std::{convert::Infallible, fmt::Debug};

use super::{Action, ActionContext, ActionResult};

// TODO: add error assoc type.
// Right now no built-in provided filters can error but a user-implemented one might
/// Trait for all types that support filtering entries out of a list of [`Entry`]s
pub trait Filter: Debug + MaybeSendSync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Error: Into<FilterError>;

	/// Filter or modify the list of entries
	fn filter(
		&self,
		entries: &mut Vec<Entry>,
	) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend;
}

pub(crate) struct FilterWrapper<F>(pub F);

impl<F> Action for FilterWrapper<F>
where
	F: Filter,
{
	type Error = FilterError;

	async fn apply<S, E>(
		&mut self,
		mut entries: Vec<Entry>,
		_ctx: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Error> {
		match self.0.filter(&mut entries).await {
			Ok(()) => ActionResult::Ok(entries),
			Err(e) => ActionResult::Err(e.into()),
		}
	}
}

impl Filter for () {
	type Error = Infallible;

	async fn filter(&self, _entries: &mut Vec<Entry>) -> Result<(), Self::Error> {
		Ok(())
	}
}

impl<F: Filter> Filter for Option<F> {
	type Error = F::Error;

	async fn filter(&self, entries: &mut Vec<Entry>) -> Result<(), Self::Error> {
		let Some(f) = self else {
			return Ok(());
		};

		f.filter(entries).await
	}
}

impl Filter for Infallible {
	type Error = Infallible;

	async fn filter(&self, _entries: &mut Vec<Entry>) -> Result<(), Self::Error> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl Filter for ! {
	type Error = !;

	async fn filter(&self, _entries: &mut Vec<Entry>) -> Result<(), Self::Error> {
		match *self {}
	}
}
