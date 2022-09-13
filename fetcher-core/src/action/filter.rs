/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod take;

pub use take::Take;

use super::regex::{action::Find, Regex};
use crate::{entry::Entry, read_filter::ReadFilter};

use derive_more::From;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait Filter {
	fn filter(&self, entries: &mut Vec<Entry>);
}

#[derive(From, Debug)]
pub enum Kind {
	ReadFilter(Arc<RwLock<ReadFilter>>),
	Take(Take),
	Regex(Regex<Find>),
}

impl Kind {
	pub async fn filter(&self, entries: &mut Vec<Entry>) {
		match self {
			Kind::ReadFilter(rf) => rf.read().await.filter(entries),
			Kind::Take(x) => x.filter(entries),
			Kind::Regex(x) => x.filter(entries),
		}
	}
}
