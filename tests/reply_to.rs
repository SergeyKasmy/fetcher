/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This test asserts that the message id passed to the sink is the correct
//! message id that corresponds to the entry that the source asked to be replied to

#![allow(clippy::missing_assert_message)]
#![allow(clippy::tests_outside_test_module)]
#![allow(clippy::unwrap_used)]

use std::{convert::Infallible, sync::LazyLock};

use fetcher::{
	actions::sink,
	entry::{Entry, EntryId},
	error::FetcherError,
	read_filter::MarkAsRead,
	sinks::{
		Sink,
		message::{Message, MessageId},
	},
	sources::{Fetch, Source},
	task::{Task, entry_to_msg_map::EntryToMsgMap},
};

static ENTRY_ID: LazyLock<EntryId> = LazyLock::new(|| EntryId::new("0".to_owned()).unwrap());
const MESSAGE_ID: i64 = 0;

#[derive(Debug)]
struct DummySource;

#[derive(Debug)]
struct DummySink;

impl Fetch for DummySource {
	type Err = Infallible;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		Ok(vec![Entry {
			reply_to: Some(ENTRY_ID.clone()),
			..Default::default()
		}])
	}
}

impl MarkAsRead for DummySource {
	async fn mark_as_read(&mut self, _id: &EntryId) -> Result<(), FetcherError> {
		Ok(())
	}

	async fn set_read_only(&mut self) {}
}

impl Source for DummySource {}

impl Sink for DummySink {
	type Err = Infallible;

	async fn send(
		&mut self,
		_message: &Message,
		reply_to: Option<&MessageId>,
		_tag: Option<&str>,
	) -> Result<Option<MessageId>, Self::Err> {
		assert_eq!(reply_to.unwrap().0, MESSAGE_ID);

		Ok(None)
	}
}

#[tokio::test]
async fn reply_to() {
	let mut entry_to_msg_map = EntryToMsgMap::<()>::default();

	entry_to_msg_map
		.insert(ENTRY_ID.to_owned(), MESSAGE_ID.into())
		.await
		.unwrap();

	let mut task = Task::builder("reply_to_test")
		.source(DummySource)
		.action(sink(DummySink))
		.entry_to_msg_map(entry_to_msg_map)
		.build();

	task.run().await.unwrap();
}
