/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This example showcases a basic [`ExternalSave`] implementation that just saves
//! serialized as JSON read-filter state to a file on the filesystem,
//! as well as the expected way to use a [`ReadFilter`].

use std::{any::type_name, error::Error, io, iter, path::PathBuf};

use fetcher::{
	Job,
	actions::{filter, filters::Filter},
	external_save::{ExternalSave, ExternalSaveError},
	job::trigger,
	read_filter::{MarkAsRead, Newer, ReadFilter},
	sources::Fetch,
};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
	fs::{self, File},
	io::{AsyncSeekExt, AsyncWriteExt},
	sync::OnceCell,
};

/// A truncating file writer, where each write call completely replaces the previous file contents with the new ones.
///
/// It implements [`ExternalSave`] by serializing the read-filter to JSON and writing it to the filesystem.
#[derive(Debug)]
pub struct TruncatingFileWriter {
	/// Path of the file
	path: PathBuf,
	/// File handle, created on demand
	file: OnceCell<File>,
}

impl TruncatingFileWriter {
	fn new(path: PathBuf) -> Self {
		Self {
			path,
			file: OnceCell::new(),
		}
	}

	/// Writes `s` to the file, replacing all previous contents
	async fn write(&mut self, s: &str) -> Result<(), io::Error> {
		// Create the file and its parent directories.
		// Sadly .get_or_try_init returns a shared reference, so we have to manually call .get_mut later
		self.file
			.get_or_try_init(async || {
				let parent = self.path.parent().unwrap();
				fs::create_dir_all(parent).await?;

				fs::OpenOptions::new()
					.create(true)
					.truncate(true)
					.write(true)
					.open(&self.path)
					.await
			})
			.await?;

		let file = self
			.file
			.get_mut()
			.expect("Should've been initialized just right up above");

		// truncate the wile
		file.set_len(0).await?;
		file.rewind().await?; // TODO: not sure if necessary

		// write the string and flush
		file.write_all(s.as_bytes()).await?;
		file.flush().await?;

		Ok(())
	}
}

impl ExternalSave for TruncatingFileWriter {
	async fn save_read_filter<RF>(&mut self, read_filter: &RF) -> Result<(), ExternalSaveError>
	where
		RF: Serialize,
	{
		// serialize the read filter to JSON
		let rf_as_json =
			serde_json::to_string(read_filter).expect("RF serialization should not fail");

		// and write it to the file
		self.write(&rf_as_json)
			.await
			.map_err(|source| ExternalSaveError {
				source,
				path: Some(self.path.to_string_lossy().into_owned()),
			})
	}

	// this function can be implemented similarly
	async fn save_entry_to_msg_map(
		&mut self,
		_map: &std::collections::HashMap<
			fetcher::entry::EntryId,
			fetcher::sinks::message::MessageId,
		>,
	) -> std::result::Result<(), ExternalSaveError> {
		todo!("do something similar to save_read_filter")
	}
}

/// Reads and deserializes a specific read-filter implementation from the filesystem
pub fn rf_from_file<RF>(
	job_group: Option<&'static str>,
	job_name: &'static str,
	task_name: Option<&'static str>,
) -> ReadFilter<RF, true, TruncatingFileWriter>
where
	RF: DeserializeOwned + MarkAsRead + Filter + Default + Serialize,
{
	// Construct the path "job_group/job/task" where each missing path part is just skipped.
	// In our case we have no job group or task name, so it'll just be a file named the same as the job is.
	let rf_path = iter::once("read_filter")
		.chain(job_group)
		.chain(iter::once(job_name))
		.chain(task_name)
		.collect::<PathBuf>();

	// Read the file and deserialize it.
	// Create an empty read-filter if the file doesn't exist
	let rf = match std::fs::read_to_string(&rf_path) {
		Ok(file_contents) => serde_json::from_str(&file_contents).unwrap_or_else(|_| {
			panic!(
				"Couldn't deserialize read filter (type {}) from file at {}",
				type_name::<RF>(),
				rf_path.display()
			)
		}),
		Err(e) if e.kind() == io::ErrorKind::NotFound => RF::default(),
		Err(e) => {
			panic!(
				"RF file couldn't be read at {}: {e}",
				rf_path.to_string_lossy().into_owned()
			);
		}
	};

	// combine the deserialized read-filter with the truncating file writer external saver into a single shared read-filter
	ReadFilter::new(rf, TruncatingFileWriter::new(rf_path))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
	const JOB_NAME: &str = "example";

	// read the previous read-filter state from the filesystem
	let read_filter = rf_from_file::<Newer>(None, JOB_NAME, None);

	// create a job that uses the read filter to mark entries as read in the source and filters read entries in the action
	let mut job = Job::builder_simple(JOB_NAME)
		.source(String::from("Hello, World").into_source_with_read_filter(read_filter.clone())) // < here it's used to mark entries as read
		.action((
			// TODO: somehow parse the input
			filter(read_filter), // < here it's used to filter out already read entries
		))
		.cancel_token(None)
		.trigger(trigger::Never)
		.build_with_default_error_handling();

	job.run().await.unwrap();

	Ok(())
}
