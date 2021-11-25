use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewsReaderError {
	#[error("{service} authentication error: {why}")]
	Auth { service: String, why: String },
	#[error("error getting data from {service}: {why}")]
	Get { service: String, why: String },
	#[error("error parsing data from {service}: {why}")]
	Parse { service: String, why: String },
	#[error("error sending data to Telegram: {why}")]
	r#Send { why: String },
	#[error("error saving last read item's GUID to disk: {why}")]
	GuidSave { why: String },
	//#[error(transparent)]
	//Save(#[from] std::io::Error),
	//#[error("error retrieving data from RSS feed {feed}: {why}")]
	//RssGet { feed: &'static str, why: String },
	//#[error("error parsing data from RSS feed {feed}: {why}")]
	//RssParse { feed: &'static str, why: String },
	//#[error("error retrieving data from Twitter {handle}: {why}")]
	//Twitter { handle: &'static str, why: String },
	//#[error("error sending news to Telegram: {0}")]
	//Telegram(String),
}

pub type Result<T> = std::result::Result<T, NewsReaderError>;
