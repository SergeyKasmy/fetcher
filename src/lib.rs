mod error;
pub(crate) mod guid;
mod rss_news_reader;
mod twitter_news_reader;

pub use rss_news_reader::RssNewsReader;
pub use twitter_news_reader::TwitterNewsReader;
