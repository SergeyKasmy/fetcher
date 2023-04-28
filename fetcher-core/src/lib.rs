/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
// Additional Lints
#![warn(clippy::pedantic)]
// some types are more descriptive with modules name in the name, especially if this type is often used out of the context of this module
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::nursery)]
#![allow(clippy::option_if_let_else)] // "harder to read, false branch before true branch"
#![allow(clippy::use_self)] // may be hard to understand what Self even is deep into a function's body
#![allow(clippy::equatable_if_let)] // matches!() adds too much noise for little benefit

pub mod action;
pub mod auth;
pub mod entry;
pub mod error;
mod exec;
pub mod external_save;
pub mod job;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;
pub mod utils;
