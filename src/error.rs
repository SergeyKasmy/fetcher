#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("error getting program data: {0}")]
	GetData(String),
	#[error("error saving program data: {0}")]
	SaveData(String),
	#[error("{service} authentication error: {why}")]
	Auth { service: String, why: String },
	#[error("error getting data from {service}: {why}")]
	Fetch { service: String, why: String },
	#[error("error parsing data from {service}: {why}")]
	Parse { service: String, why: String },
	#[error("error sending data to Telegram: {0}")]
	r#Send(String),
}

pub type Result<T> = std::result::Result<T, Error>;
