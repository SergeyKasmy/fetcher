/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::FetcherError;

pub trait RunResult: IntoIterator<Item = Result<(), FetcherError>> {}

impl<const N: usize> RunResult for [Result<(), FetcherError>; N] {}
impl RunResult for Vec<Result<(), FetcherError>> {}
impl RunResult for std::iter::Once<Result<(), FetcherError>> {}
