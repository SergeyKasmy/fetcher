/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// #![cfg_attr(doc, doc = include_str!("../README.md"))]
// TEMP
#![allow(missing_docs)]
#![allow(async_fn_in_trait)]
// TODO: enable later
#![allow(clippy::missing_docs_in_private_items)]
// TODO: add a "send" feature that makes all futures send
#![allow(clippy::future_not_send)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

mod static_str;

pub mod action;
pub mod auth;
pub mod ctrl_c_signal;
pub mod entry;
pub mod error;
pub mod exec;
pub mod external_save;
pub mod job;
pub mod read_filter;
pub mod sinks;
pub mod sources;
pub mod task;
pub mod utils;

#[cfg(feature = "scaffold")]
pub mod scaffold;

pub use crate::static_str::StaticStr;

pub use either;
pub use url;
