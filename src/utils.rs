/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Miscellaneous utility extention traits for external types

pub mod display_debug;
pub mod option_ext;

pub use self::{display_debug::DisplayDebug, option_ext::OptionExt};
