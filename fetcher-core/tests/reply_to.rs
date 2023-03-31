//! This test asserts that the message id passed to the sink is the correct
//! message id that corresponds to the entry that the source asked to be replied to

use async_trait::async_trait;
use fetcher_core::{
	entry::{Entry, EntryId},
	error::Error,
	read_filter::MarkAsRead,
	sink::{
		error::SinkError,
		message::{Message, MessageId},
		Sink,
	},
	source::{error::SourceError, Fetch, Source},
	task::{entry_to_msg_map::EntryToMsgMap, Task},
};

const ENTRY_ID: &str = "0";
const MESSAGE_ID: i64 = 0;

#[derive(Debug)]
struct DummySource;

#[derive(Debug)]
struct DummySink;

#[async_trait]
impl Fetch for DummySource {
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		Ok(vec![Entry {
			reply_to: Some(EntryId(ENTRY_ID.into())),
			..Default::default()
		}])
	}
}

#[async_trait]
impl MarkAsRead for DummySource {
	async fn mark_as_read(&mut self, _id: &EntryId) -> Result<(), Error> {
		Ok(())
	}

	async fn set_read_only(&mut self) {}
}

impl Source for DummySource {}

#[async_trait]
impl Sink for DummySink {
	async fn send(
		&self,
		_message: Message,
		reply_to: Option<&MessageId>,
		_tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError> {
		assert_eq!(reply_to.unwrap().0, MESSAGE_ID);
		Ok(None)
	}
}

#[tokio::test]
async fn reply_to() {
	let mut entry_to_msg_map = EntryToMsgMap::default();

	entry_to_msg_map
		.insert(ENTRY_ID.to_owned().into(), MESSAGE_ID.into())
		.await
		.unwrap();

	let mut task = Task {
		tag: None,
		source: Some(Box::new(DummySource)),
		actions: None,
		sink: Some(Box::new(DummySink)),
		entry_to_msg_map: Some(entry_to_msg_map),
	};

	task.run().await.unwrap();
}
