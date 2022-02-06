pub mod email;
pub mod rss;
pub mod twitter;

pub use self::email::Email;
pub use self::rss::Rss;
pub use self::twitter::Twitter;

use crate::error::Result;
use crate::sink::Message;

pub struct Responce {
	pub id: Option<String>,
	pub msg: Message,
}

#[derive(Debug)]
pub enum Source {
	Email(Email),
	Rss(Rss),
	Twitter(Twitter),
}

impl Source {
	// TODO: try using streams instead of polling manually?
	pub async fn get(&mut self, last_read_id: Option<String>) -> Result<Vec<Responce>> {
		match self {
			Self::Email(x) => x.get().await,
			Self::Rss(x) => x.get(last_read_id).await,
			Self::Twitter(x) => x.get(last_read_id).await,
		}
	}
}

impl From<Email> for Source {
	fn from(e: Email) -> Self {
		Self::Email(e)
	}
}

impl From<Rss> for Source {
	fn from(r: Rss) -> Self {
		Self::Rss(r)
	}
}

impl From<Twitter> for Source {
	fn from(t: Twitter) -> Self {
		Self::Twitter(t)
	}
}
