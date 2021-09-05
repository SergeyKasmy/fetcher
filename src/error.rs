use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewsReaderError {
	#[error("{0} authentication error: {why}")]
	Auth { service: &'static str, why: String },
	#[error(transparent)]
	Save(#[from] std::io::Error),
	#[error("error retrieving data from RSS feed {feed}: {why}")]
	RssGet { feed: &'static str, why: String },
	#[error("error parsing data from RSS feed {feed}: {why}")]
	RssParse { feed: &'static str, why: String },
	#[error("error retrieving data from Twitter {handle}: {why}")]
	Twitter { handle: &'static str, why: String },
	#[error("error sending news to Telegram: {0}")]
	Telegram(String),
}

pub type Result<T> = std::result::Result<T, NewsReaderError>;
