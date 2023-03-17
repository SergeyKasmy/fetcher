/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]

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
