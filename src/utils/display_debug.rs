/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`DisplayDebug`] trait

use std::fmt;

/// A combined trait that requires to implement both Display and Debug
pub trait DisplayDebug: fmt::Display + fmt::Debug {}

impl<T: fmt::Display + fmt::Debug> DisplayDebug for T {}
