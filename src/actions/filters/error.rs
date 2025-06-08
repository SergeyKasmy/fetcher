/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`FilterError`] type

use crate::error::{Error, error_trait::BoxErrorWrapper};

use std::{convert::Infallible, error::Error as StdError};

/// An error that occured during filtering of entries
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum FilterError {
	#[error("Other error")]
	Other(Box<dyn Error>),
}

impl Error for FilterError {
	fn is_network_related(&self) -> Option<&dyn Error> {
		match self {
			Self::Other(other_err) if other_err.is_network_related().is_some() => Some(self),
			_ => None,
		}
	}
}

impl From<Box<dyn StdError + Send + Sync>> for FilterError {
	fn from(value: Box<dyn StdError + Send + Sync>) -> Self {
		Self::Other(Box::new(BoxErrorWrapper(value)))
	}
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
