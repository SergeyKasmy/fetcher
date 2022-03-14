/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)] // TODO
#![allow(clippy::missing_errors_doc)]

// TODO: more tests

pub mod auth;
pub mod config;
pub mod entry;
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
use crate::source::Source;
use crate::task::Task;

pub async fn run_task(name: &str, t: &mut Task) -> Result<()> {
	let mut read_filter = t
		.read_filter_kind
		.map(|x| ReadFilter::read_from_fs(name.to_owned(), x))
		.transpose()?;

	loop {
		tracing::trace!("Running...");

		let fetch = async {
			for entry in t.source.get(read_filter.as_ref()).await? {
				tracing::trace!("Processing entry: {entry:?}");

				t.sink.send(entry.msg).await?;
				match (&mut t.source, &mut read_filter) {
					// Email has custom read filtering and read marking
					(Source::Email(e), None) => e.mark_as_read(&entry.id).await?,
					// delete read_filter save file if it was created for some very strange reason for this source type
					(Source::Email(_), Some(_)) => read_filter.take().unwrap().delete_from_fs()?,
					(_, Some(f)) => f.mark_as_read(&entry.id)?,
					_ => unreachable!(),
				}
			}

			Ok::<(), Error>(())
		};

		match fetch.await {
			Ok(_) => (),
			Err(e @ Error::Network(_)) => tracing::warn!("{:?}", anyhow::anyhow!(e)),
			Err(e) => return Err(e),
		}

		tracing::debug!("Sleeping for {time}m", time = t.refresh);
		sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
	}
}
