/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{error::Error, fmt::Write};

/// Extention trait for [`std::error::Error`] to print the entire chain of the error
pub trait ErrorChainExt {
	/// Return a string intented for logging or printing that formats an error's entire error source chain
	// #[deprecated = "Use eyre instead"]
	fn display_chain(&self) -> String;
}

impl<T: Error> ErrorChainExt for T {
	#[must_use]
	fn display_chain(&self) -> String {
		let mut current_err: &dyn Error = self;
		let mut counter = 0;
		let mut output = format!("{current_err}");

		while let Some(source) = Error::source(current_err) {
			current_err = source;
			counter += 1;
			if counter == 1 {
				_ = write!(output, "\n\nCaused by:");
			}

			_ = write!(output, "\n\t{counter}: {current_err}");
		}

		output
	}
}
