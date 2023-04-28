/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::time::Instant;

/// With the default `2^err_num` mins sleeping stradegy, this will sleep for 5.6 days
/// which is more than enough to be sure that the job is broken and should just be stopped
pub const DEFAULT_MAX_ERROR_LIMIT: u32 = 13;

#[derive(Clone, Debug)]
pub enum ErrorHandling {
	Forward,
	LogAndIgnore,
	Sleep { prev_errors: PrevErrors },
}

/// This keeps count of how many errors have happened,
/// the time the last error has happened,
/// and what's the maximum amount of errors allowed before it's too much
#[derive(Clone, Debug)]
pub struct PrevErrors {
	pub max_retries: u32,

	err_count: u32,
	last_error: Option<Instant>,
}

impl PrevErrors {
	#[must_use]
	pub const fn new(max_retries: u32) -> Self {
		Self {
			max_retries,
			err_count: 0,
			last_error: None,
		}
	}

	/// Returns true if max error limit reached
	pub fn push(&mut self) -> bool {
		self.err_count += 1;

		if self.err_count >= self.max_retries {
			return true;
		}

		self.last_error = Some(Instant::now());

		false
	}

	pub fn reset(&mut self) {
		self.err_count = 0;
		self.last_error = None;
	}

	#[must_use]
	pub const fn count(&self) -> u32 {
		self.err_count
	}

	#[must_use]
	pub const fn last_error(&self) -> Option<&Instant> {
		self.last_error.as_ref()
	}
}
