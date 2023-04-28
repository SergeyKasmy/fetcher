/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![doc = include_str!("../README.md")]
// Hand selected lints
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
#![forbid(unsafe_code)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::exit)]
#![warn(clippy::filetype_is_file)]
#![warn(clippy::format_push_string)]
#![warn(clippy::let_underscore_untyped)]
#![warn(clippy::missing_assert_message)]
// #![warn(clippy::missing_docs_in_private_items)]	// TODO: enable later
#![warn(clippy::print_stderr)]
#![warn(clippy::rest_pat_in_fully_bound_structs)]
#![warn(clippy::same_name_method)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_to_string)]
#![warn(clippy::tests_outside_test_module)]
#![warn(clippy::todo)]
#![warn(clippy::try_err)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unimplemented)]
// Additional automatic Lints
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
