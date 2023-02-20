/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Filter`] trait that can be implemented in filters and the [`Take`] filter, as well as an enum of all available filters

pub mod take;

use async_trait::async_trait;
pub use take::Take;

use super::regex::{action::Find, Regex};
use crate::{entry::Entry, read_filter::ReadFilter};

use derive_more::From;
use std::{fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

/// Helper trait for all types that support filtering entries out of a list of [`Entry`]s
#[async_trait]
pub trait Filter: Debug {
	async fn filter(&self, entries: &mut Vec<Entry>);
}

// FIXME: implement on the ReadFilter type itself
#[derive(Debug)]
pub struct ReadFilterAction(pub Arc<RwLock<ReadFilter>>);

#[async_trait]
impl Filter for ReadFilterAction {
	async fn filter(&self, entries: &mut Vec<Entry>) {
		self.0.read().await.filter(entries);
	}
}

/*
/// A list of all available filters
#[derive(From, Debug)]
pub enum Kind {
	/// Filter out read entries
	ReadFilter(Arc<RwLock<ReadFilter>>),
	/// Take a certain number of entries and filter out the rest
	Take(Take),
	/// Leave in only the entries that match the regular expr and filter out the rest
	Regex(Regex<Find>),
}

impl Kind {
	/// Calls each enum variant's [`Filter::filter()`] impl
	// This type doesn't implement Filter trait itself since the Read Filter requires async locking
	// and there's no reason to add the overhead of a Box'ed future type (via #[async_trait]) just for that one impl.
	// If more transforms will require async in the future, I may as well make Filter async and implement it for Kind
	pub async fn filter(&self, entries: &mut Vec<Entry>) {
		match self {
			Kind::ReadFilter(rf) => rf.read().await.filter(entries),
			Kind::Take(x) => x.filter(entries),
			Kind::Regex(x) => x.filter(entries),
		}
	}
}
*/
