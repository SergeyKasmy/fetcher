#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("can't read program config: {0}")]
	GetConfig(String),
	#[error("can't read program data: {0}")]
	GetData(String),
	#[error("can't save program data: {0}")]
	SaveData(String),
	#[error("env var not found: {0}")]
	GetEnvVar(String),
	#[error("invalid config format")]
	ConfigDeserialize(#[from] toml::de::Error),
	#[error("{name} is missing {field} field")]
	ConfigMissingField { name: String, field: &'static str },
	#[error("{name}'s {field} field is not a valid {expected_type}")]
	ConfigInvalidFieldType { name: String, field: &'static str, expected_type: &'static str },
	#[error("{service} authentication error: {why}")]
	SourceAuth { service: String, why: String },
	#[error("can't fetch data from {service}: {why}")]
	SourceFetch { service: String, why: String },
	#[error("can't parse data from {service}: {why}")]
	SourceParse { service: String, why: String },
	#[error("can't send data to {where_to}: {why}")]
	SinkSend { where_to: String, why: String },
}

pub type Result<T> = std::result::Result<T, Error>;
