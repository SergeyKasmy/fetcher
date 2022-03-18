/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub mod named_task;
pub mod task;
pub mod template;

pub use self::{
	named_task::NamedTask,
	task::{Task, Tasks},
	template::Template,
};
