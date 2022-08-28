/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core as fcore;

use std::collections::HashMap;

pub(crate) type Tasks = HashMap<String, Task>;
pub(crate) struct Task {
	pub inner: fcore::task::Task,
	pub refresh: u64,
}
