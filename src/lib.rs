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
use crate::read_filter::ReadFilter;
use crate::task::Task;

#[tracing::instrument(skip(t))]
pub async fn run_task(name: &str, t: &mut Task) -> Result<()> {
	let mut read_filter = ReadFilter::read_from_fs(name)?;
	loop {
		tracing::debug!("Fetching");

		let fetch = async {
			for rspn in t.source.get(&read_filter).await? {
				t.sink.send(rspn.msg).await?;

				if let Some(id) = rspn.id {
					read_filter.mark_as_read(&id);
				}
			}

			Ok::<(), Error>(())
		};

		match fetch.await {
			Ok(_) => (),
			Err(e @ Error::Network(_)) => tracing::warn!("{e:?}"),
			Err(e) => return Err(e),
		}

		tracing::debug!("Sleeping for {time}m", time = t.refresh);
		sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
	}
}
