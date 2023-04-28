/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: mb rename .parse() into .into() or something of that sort? .into() is already used by From/Into traits though. Naming is hard, man... UPD: into_conf() and from_conf() are way better!

#![doc = include_str!("../README.md")]
#![warn(clippy::unwrap_used)]
// Additional Lints
#![warn(clippy::pedantic)]
// some types are more descriptive with modules name in the name, especially if this type is often used out of the context of this module
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::nursery)]
#![allow(clippy::option_if_let_else)] // "harder to read, false branch before true branch"
#![allow(clippy::use_self)] // may be hard to understand what Self even is deep into a function's body
#![allow(clippy::equatable_if_let)] // matches!() adds too much noise for little benefit
#![allow(clippy::missing_const_for_fn)] // most of methods take self and self destructor can't be const, so this is pretty much iseless
#![allow(clippy::missing_errors_doc)] // TODO: add more docs

pub mod error;
pub mod jobs;
mod serde_extentions;
pub mod settings;

pub use self::error::Error;
