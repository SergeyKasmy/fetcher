/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod config;
pub mod data;
pub mod external_data;
pub mod read_filter;

use once_cell::sync::OnceCell;
use std::path::PathBuf;

const PREFIX: &str = "fetcher";

pub static DATA_PATH: OnceCell<PathBuf> = OnceCell::new();
pub static CONF_PATHS: OnceCell<Vec<PathBuf>> = OnceCell::new();
