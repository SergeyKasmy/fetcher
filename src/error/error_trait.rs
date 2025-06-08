/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error as StdError;

// TODO: better name??
// TODO: use maybesendsync?
/// A subtrait of [`std::error::Error`] that requires specifying if the error is somehow network-related
pub trait Error: StdError + Send + Sync {
	/// Checks if the current error is somehow related to the network connection and return the error if it is.
	///
	/// Usually these errors shouldn't contribute to error handling in jobs and should just sleep for a bit before retrying.
	#[must_use]
	fn is_network_related(&self) -> Option<&dyn Error>;
}

impl StdError for Box<dyn Error> {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		(**self).source()
	}
}

// make a wrapper around Box<dyn StdError> and implement Error and StdError for it
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub(crate) struct BoxErrorWrapper(pub Box<dyn StdError + Send + Sync>);

impl Error for BoxErrorWrapper {
	// assume an opaque error is not network related
	fn is_network_related(&self) -> Option<&dyn Error> {
		None
	}
}

impl From<Box<dyn StdError + Send + Sync>> for Box<dyn Error> {
	fn from(value: Box<dyn StdError + Send + Sync>) -> Self {
		Box::new(BoxErrorWrapper(value))
	}
}
