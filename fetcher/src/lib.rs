/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod extentions;
pub mod settings;

use fetcher_config::jobs::named::{JobName, JobWithTaskNames};

use std::collections::HashMap;

// TODO: use a BTreeMap and avoid sorting in .display()?
pub type Jobs = HashMap<JobName, JobWithTaskNames>;
