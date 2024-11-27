/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![doc = include_str!("../README.md")]
#![allow(clippy::missing_docs_in_private_items)] // TODO: enable later
#![allow(clippy::missing_errors_doc)] // TODO: add more docs
#![allow(missing_docs)] // TODO: add more docs

pub mod error;
pub mod jobs;
mod serde_extentions;
pub mod settings;

pub use self::error::FetcherConfigError;
