/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{
	error::Error as StdError,
	fmt::{self, Display},
};

/// Wrapper around a type implementing [`std::error::Error`]
/// that provides a pretty [`Display`] implementation.
///
/// It may looked like this:
///
/// Error 1
///
/// Caused by:
///   1: Error 2
///   2: Error 3
///   3: Error 4
pub struct ErrorChainDisplay<'a>(pub &'a dyn StdError);

impl Display for ErrorChainDisplay<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut current_err = self.0;
		let mut counter = 0;
		write!(f, "{current_err}")?;

		while let Some(source) = StdError::source(current_err) {
			current_err = source;
			counter += 1;
			if counter == 1 {
				write!(f, "\n\nCaused by:")?;
			}

			write!(f, "\n\t{counter}: {current_err}")?;
		}

		Ok(())
	}
}
