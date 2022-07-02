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
#![allow(clippy::module_name_repetitions)]

// TODO: more tests

pub mod auth;
pub mod config;
pub mod entry;
pub mod error;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;

use sink::Sink;
use source::Source;

use crate::entry::Entry;
use crate::error::Error;
use crate::error::Result;
use crate::task::Task;

pub async fn run_task(t: &mut Task) -> Result<()> {
	tracing::trace!("Running task...");

	let fetch = async {
		for entry in t.source.get(t.parsers.as_deref()).await? {
			process_entry(&mut t.sink, entry, t.tag.as_deref(), &mut t.source).await?;
		}

		Ok::<(), Error>(())
	};

	match fetch.await {
		Ok(_) => (),
		Err(e @ Error::NoConnection(_)) => tracing::warn!("{:?}", color_eyre::eyre::eyre!(e)),
		Err(e) => return Err(e),
	}

	Ok(())
}

#[tracing::instrument(name = "entry", skip_all, fields(id = entry.id.as_str()))]
async fn process_entry(
	sink: &mut Sink,
	entry: Entry,
	tag: Option<&str>,
	mark_as_read: &mut Source,
) -> Result<()> {
	tracing::trace!("Processing entry: {entry:?}");

	sink.send(entry.msg, tag).await?;
	mark_as_read.mark_as_read(&entry.id).await?;

	Ok::<(), Error>(())
}
