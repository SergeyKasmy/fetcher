/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![doc = include_str!("../README.md")]
#![allow(clippy::missing_docs_in_private_items)] // TODO: enable later

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
