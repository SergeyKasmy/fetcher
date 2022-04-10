pub mod html;
pub mod rss;

pub use self::html::Html;
pub use self::rss::Rss;

use crate::entry::Entry;
use crate::error::Result;

#[derive(Debug)]
pub enum Parser {
	Html(Html),
	Rss(Rss),
}

impl Parser {
	pub async fn parse(&self, entries: Vec<Entry>) -> Result<Vec<Entry>> {
		match self {
			Parser::Html(x) => x.process(entries).await,
			Parser::Rss(x) => x.parse(entries),
		}
	}
}
