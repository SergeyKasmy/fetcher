/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`OptionExt`] trait

/// Alternative failable methods on [`Option`]
#[expect(
	clippy::missing_errors_doc,
	reason = "function signature is self-explaining"
)]
pub trait OptionExt<T> {
	/// [`Option::map()`] alternative that can return a result
	fn try_map<U, E, F>(self, f: F) -> Result<Option<U>, E>
	where
		F: FnOnce(T) -> Result<U, E>;

	/// [`Option::and_then()`] alternative that can return a result
	fn try_and_then<U, E, F>(self, f: F) -> Result<Option<U>, E>
	where
		F: FnOnce(T) -> Result<Option<U>, E>;
}

impl<T> OptionExt<T> for Option<T> {
	fn try_map<U, E, F>(self, f: F) -> Result<Option<U>, E>
	where
		F: FnOnce(T) -> Result<U, E>,
	{
		match self {
			Some(x) => f(x).map(Some),
			None => Ok(None),
		}
	}

	fn try_and_then<U, E, F>(self, f: F) -> Result<Option<U>, E>
	where
		F: FnOnce(T) -> Result<Option<U>, E>,
	{
		match self {
			Some(x) => f(x),
			None => Ok(None),
		}
	}
}
