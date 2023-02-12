/*
 * this source code form is subject to the terms of the mozilla public
 * license, v. 2.0. if a copy of the mpl was not distributed with this
 * file, you can obtain one at https://mozilla.org/mpl/2.0/.
 */

use fetcher_core::{
	self as fcore,
	read_filter::{Kind as ReadFilterKind, ReadFilter},
};

use std::{
	fmt::{Debug, Display},
	io,
	path::Path,
};
use thiserror::Error;

pub enum ExternalDataResult<T, E = ExternalDataError> {
	Ok(T),
	Unavailable,
	Err(E),
}

pub trait ProvideExternalData {
	fn twitter_token(&self) -> ExternalDataResult<(String, String)>;
	fn google_oauth2(&self) -> ExternalDataResult<fcore::auth::Google>;
	fn email_password(&self) -> ExternalDataResult<String>;
	fn telegram_bot_token(&self) -> ExternalDataResult<String>;
	fn read_filter(
		&self,
		name: &str,
		expected_rf: ReadFilterKind,
	) -> ExternalDataResult<ReadFilter>;
}

#[derive(Error, Debug)]
pub enum ExternalDataError {
	#[error("IO error{}", if let Some(p) = payload { format!(": {p}") } else { String::new() })]
	Io {
		source: io::Error,
		payload: Option<Box<dyn DisplayDebug + Send + Sync>>,
	},
	#[error("Incompatible read filter types: in config: \"{expected}\" and found: \"{found}\"{}", if let Some(p) = payload { format!(": {p}") } else { String::new() })]
	ReadFilterIncompatibleTypes {
		expected: ReadFilterKind,
		found: ReadFilterKind,
		payload: Option<Box<dyn DisplayDebug + Send + Sync>>,
	},
}

impl<T, E> From<Result<T, E>> for ExternalDataResult<T, E> {
	fn from(v: Result<T, E>) -> Self {
		match v {
			Ok(v) => ExternalDataResult::Ok(v),
			Err(e) => ExternalDataResult::Err(e),
		}
	}
}

impl ExternalDataError {
	pub fn new_io_with_path(io_err: io::Error, path: &Path) -> Self {
		Self::Io {
			source: io_err,
			payload: Some(Box::new(format!("path: {}", path.to_string_lossy()))),
		}
	}

	pub fn new_rf_incompat_with_path(
		expected: ReadFilterKind,
		found: ReadFilterKind,
		path: &Path,
	) -> Self {
		Self::ReadFilterIncompatibleTypes {
			expected,
			found,
			payload: Some(Box::new(format!("path: {}", path.to_string_lossy()))),
		}
	}
}

impl From<io::Error> for ExternalDataError {
	fn from(io_err: io::Error) -> Self {
		Self::Io {
			source: io_err,
			payload: None,
		}
	}
}

impl<E, P> From<(E, P)> for ExternalDataError
where
	E: Into<io::Error>,
	P: AsRef<Path>,
{
	fn from((io_err, path): (E, P)) -> Self {
		Self::new_io_with_path(io_err.into(), path.as_ref())
	}
}

pub trait DisplayDebug: Display + Debug {}
impl<T: Display + Debug> DisplayDebug for T {}
