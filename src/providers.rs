pub mod email;
pub mod rss;
pub mod twitter;

pub use self::email::Email;
pub use self::rss::Rss;
pub use self::twitter::Twitter;

use crate::error::Result;
use crate::telegram::Message;

pub enum Provider {
	Email(Email),
	Rss(Rss),
	Twitter(Twitter),
}

impl Provider {
	pub async fn get(&mut self) -> Result<Vec<Message>> {
		match self {
    		Provider::Email(x) => x.get(),
    		Provider::Rss(x) => x.get().await,
    		Provider::Twitter(x) => x.get().await,
		}
	}
}
