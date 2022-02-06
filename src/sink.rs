mod message;
mod telegram;

pub use message::{Media, Message};
pub use telegram::Telegram;

use crate::error::Result;

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
