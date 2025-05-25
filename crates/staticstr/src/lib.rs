/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! [`StaticStr`] - a string type that can handle both static and owned strings.
//!
//! # Overview
//!
//! The [`StaticStr`] type is designed to optimize string handling in scenarios where most strings are static
//! (known at compile time) but some need to be dynamically generated. It internally uses a [`Cow`] to avoid
//! unnecessary allocations when working with static strings while still maintaining the flexibility to handle
//! owned strings when needed.
//!
//! # Use Cases
//!
//! - Configuration values that are usually hardcoded but sometimes need to be generated
//! - Message templates with occasional dynamic content
//! - Any situation where you frequently use `&'static str` but occasionally need `String`
//!
//! # Example
//!
//! ```rust
//! use staticstr::StaticStr;
//!
//! // Use with static strings - no allocation
//! let static_message: StaticStr = "Hello, World!".into();
//!
//! // Use with owned strings - allocates only when needed
//! let dynamic_message: StaticStr = format!("Hello, {}!", "User").into();
//!
//! // Both types can be used the same way
//! println!("{}", static_message);  // Hello, World!
//! println!("{}", dynamic_message); // Hello, User!
//! ```
//!
//! [`Cow`]: std::borrow::Cow

use std::{borrow::Cow, fmt::Display, ops::Deref};

/// A string that always has a 'static lifetime.
///
/// This makes it possible to use [`&'static str`]'s directly without allocating
/// while also allowing the use of plain regular-old [`String`]s.
/// This is most useful in places where in 99% of times a [`&'static str`] is used but sometimes a [`format!()`]'ed string may be required.
/// Technically, this could be used everywhere instead of [`String`]s but this introduces too much boilerplate and `.into()` transitions
/// that just pollute the code for little benefit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StaticStr(Cow<'static, str>);

impl StaticStr {
	#[must_use]
	pub const fn from_static_str(s: &'static str) -> Self {
		Self(Cow::Borrowed(s))
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl Deref for StaticStr {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl From<String> for StaticStr {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for StaticStr {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl Display for StaticStr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self)
	}
}

impl AsRef<str> for StaticStr {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
	}
}

impl From<StaticStr> for String {
	fn from(value: StaticStr) -> Self {
		value.0.into_owned()
	}
}

impl From<&StaticStr> for String {
	fn from(value: &StaticStr) -> Self {
		value.as_str().to_owned()
	}
}

impl Default for StaticStr {
	fn default() -> Self {
		Self(Cow::Borrowed(""))
	}
}

pub fn add(left: u64, right: u64) -> u64 {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
