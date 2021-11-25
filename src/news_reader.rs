mod rss;
mod twitter;

pub use self::rss::RssNewsReader;
pub use twitter::TwitterNewsReader;

use crate::error::Result;

#[async_trait::async_trait]
pub trait NewsReader {
	async fn start(&mut self) -> Result<()>;
}
