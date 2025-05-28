/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`FilterError`] type

use std::{convert::Infallible, error::Error};

/// An error that occured during filtering of entries
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum FilterError {
	#[error("Other error")]
	Other(Box<dyn Error + Send + Sync>),
}

impl From<Infallible> for FilterError {
	fn from(value: Infallible) -> Self {
		match value {}
	}
}

#[cfg(feature = "nightly")]
impl From<!> for FilterError {
	fn from(value: !) -> Self {
		match value {}
	}
}
