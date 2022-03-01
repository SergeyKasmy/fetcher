/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

mod config;
mod data;
mod last_read_id;

pub use self::config::config;
pub use self::data::{
	generate_google_oauth2, generate_google_password, generate_telegram, generate_twitter_auth,
	google_oauth2, google_password, telegram, twitter,
};
pub use self::last_read_id::{last_read_id, save_last_read_id};

const PREFIX: &str = "fetcher";