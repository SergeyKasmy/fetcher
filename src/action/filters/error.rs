use std::{convert::Infallible, error::Error};

#[derive(thiserror::Error, Debug)]
pub enum FilterError {
	#[error("Other error")]
	Other(Box<dyn Error>),
}

impl From<Infallible> for FilterError {
	fn from(value: Infallible) -> Self {
		match value {}
	}
}
