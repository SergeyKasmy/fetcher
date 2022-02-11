/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: maybe use a specialized crate for configs instead of serde?

pub mod email;
pub mod rss;
pub mod telegram;
pub mod twitter;

pub use self::email::Email;
pub use self::rss::Rss;
pub use self::telegram::Telegram;
pub use self::twitter::Twitter;
