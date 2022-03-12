/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: add deny_unknown_fields annotations to every config struct
// TODO: mb rename .parse() into .into() or something of that sort? .into() is already used by From/Into traits though. Naming is hard, man... UPD: into_conf() and from_conf() are way better!

pub(crate) mod auth;
pub(crate) mod read_filter;
pub(crate) mod sink;
pub(crate) mod source;
pub(crate) mod task;

pub use self::task::Task;
pub use self::task::Templates;
