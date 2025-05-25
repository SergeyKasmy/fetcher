/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Container types that guarantee non-emptiness.
//!
//! This crate provides wrapper types around `Vec` and `String` that ensure they can never be empty.
//! The name "non-non-full" is a playful way of saying "not empty" (i.e., full).
//!
//! # Features
//!
//! - `NonEmptyVec<T>`: A vector that always contains at least one element
//! - `NonEmptyString`: A string that always contains at least one character
//! - Optional serde support via the "serde" feature
//!
//! # Example
//!
//! ```
//! use non_non_full::{NonEmptyVec, NonEmptyString};
//!
//! // Creating non-empty containers
//! let vec = NonEmptyVec::new_one(42);
//! let string = NonEmptyString::new("Hello".to_string()).unwrap();
//!
//! // Operations that would make the container empty are prevented
//! let mut vec = NonEmptyVec::new(vec![1, 2]).unwrap();
//! assert_eq!(vec.pop(), Some(2)); // Allowed - vec still contains [1]
//! assert_eq!(vec.pop(), None);    // Prevented - would make vec empty
//! ```

mod string;
mod vec;

pub use self::{string::NonEmptyString, vec::NonEmptyVec};
