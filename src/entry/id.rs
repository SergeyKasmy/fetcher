/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`EntryId`] type which is used to uniquely identify entries

use non_non_full::NonEmptyString;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use tap::TapOptional;

// TODO: make generic over String/i64/other types of id
/// An ID that can identify an entry to differentiate it from another one
#[derive(PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct EntryId(pub NonEmptyString);

impl EntryId {
	/// Creates a new [`EntryId`] from the provided string, if it's not empty
	#[must_use]
	pub fn new(s: String) -> Option<Self> {
		let inner = NonEmptyString::new(s).tap_none(|| {
			tracing::warn!("Tried to create an Entry ID from an empty string");
		})?;

		Some(Self(inner))
	}

	/// Returns a string slice containing the string ID representation
	#[must_use]
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

impl Deref for EntryId {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.as_str()
	}
}

impl From<NonEmptyString> for EntryId {
	fn from(value: NonEmptyString) -> Self {
		Self(value)
	}
}

impl From<u32> for EntryId {
	fn from(value: u32) -> Self {
		Self(
			NonEmptyString::new(value.to_string())
				.expect("a number's string representation should never be empty"),
		)
	}
}

impl TryFrom<&str> for EntryId {
	// TODO: better error
	type Error = ();

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::new(value.to_owned()).ok_or(())
	}
}

impl TryFrom<String> for EntryId {
	// TODO: better error
	type Error = ();

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(value).ok_or(())
	}
}
