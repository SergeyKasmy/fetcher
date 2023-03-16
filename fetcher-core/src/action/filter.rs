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

use async_trait::async_trait;
use std::fmt::Debug;

/// Trait for all types that support filtering entries out of a list of [`Entry`]s
#[async_trait]
pub trait Filter: Debug + Send + Sync {
	/// Filter out some entries out of the `entries` vector
	async fn filter(&self, entries: &mut Vec<Entry>);
}
