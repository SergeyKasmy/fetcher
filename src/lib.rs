/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: more tests

pub mod auth;
pub mod config;
pub mod error;
pub mod read_filter;
pub mod settings;
pub mod sink;
pub mod source;
pub mod task;

use std::time::Duration;
use tokio::time::sleep;

use crate::error::Error;
use crate::error::Result;
use crate::read_filter::ReadFilterNewer;
use crate::settings::last_read_id;
use crate::settings::save_last_read_id;
use crate::task::Task;

#[tracing::instrument(skip(t))]
pub async fn run_task(name: &str, t: &mut Task) -> Result<()> {
	let mut read_filter = ReadFilterNewer {
		last_read_id: last_read_id(name)?,
	};
	loop {
		tracing::debug!("Fetching");

		match t.source.get(&read_filter).await {
			Ok(rspns) => {
				for rspn in rspns {
					t.sink.send(rspn.msg).await?;

					if let Some(id) = rspn.id {
						// save_last_read_id(name, id)?;
						read_filter.set_last_read_id(id.clone());
						save_last_read_id(name, id)?;
					}

					dbg!(&read_filter.last_read_id);
				}
			}
			Err(e) if matches!(e, Error::Network(_)) => tracing::warn!("{e}"),
			Err(e) => return Err(e),
		};

		tracing::debug!("Sleeping for {time}m", time = t.refresh);
		sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
	}
}
