pub mod email;
pub mod rss;
pub mod twitter;

pub use self::email::Email;
pub use self::rss::Rss;
pub use self::twitter::Twitter;

use crate::error::Result;
use crate::sink::Message;

#[derive(Debug)]
pub enum Source {
	Email(Email),
	Rss(Rss),
	Twitter(Twitter),
}

impl Source {
	pub async fn get(&mut self) -> Result<Vec<Message>> {
		match self {
			Self::Email(x) => x.get(),
			Self::Rss(x) => x.get().await,
			Self::Twitter(x) => x.get().await,
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
