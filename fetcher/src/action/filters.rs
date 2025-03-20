/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Filter`] trait that can be implemented in filters as well as all types that implement it

pub mod contains;
pub mod take;

pub use self::{contains::Contains, take::Take};

use crate::entry::Entry;

use std::{convert::Infallible, fmt::Debug};

use super::{Action, ActionContext};

/// Trait for all types that support filtering entries out of a list of [`Entry`]s
pub trait Filter: Debug + Send + Sync {
	/// Filter out some entries out of the `entries` vector
	async fn filter(&self, entries: &mut Vec<Entry>);

	/// Returns true if this filter is a [`ReadFilter`](crate::read_filter::ReadFilter)
	fn is_readfilter(&self) -> bool {
		false
	}
}

pub(crate) struct FilterWrapper<F>(pub F);

impl<F> Action for FilterWrapper<F>
where
	F: Filter,
{
	type Error = Infallible;

	async fn apply<S, E>(
		&mut self,
		mut entries: Vec<Entry>,
		_ctx: ActionContext<'_, S, E>,
	) -> Result<Vec<Entry>, Self::Error> {
		self.0.filter(&mut entries).await;

		Ok(entries)
	}
}

impl Filter for () {
	async fn filter(&self, _entries: &mut Vec<Entry>) {}
}
