mod telegram;

pub use telegram::Telegram;

use crate::error::Result;

pub enum Media {
	Photo(String),
	Video(String),
}

pub struct Message {
	pub text: String,
	pub media: Option<Vec<Media>>,
}

impl std::fmt::Debug for Message {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Message")
			.field("text", &self.text)
			.field("media.is_some()", &self.media.is_some())
			.finish()
	}
}

#[derive(Debug)]
pub enum Sink {
	Telegram(Telegram),
}

impl Sink {
	pub async fn send(&self, message: Message) -> Result<()> {
		match self {
			Self::Telegram(t) => t.send(message).await,
		}
	}
}
