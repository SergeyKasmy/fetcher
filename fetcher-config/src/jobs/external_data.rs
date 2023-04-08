/*
 * this source code form is subject to the terms of the mozilla public
 * license, v. 2.0. if a copy of the mpl was not distributed with this
 * file, you can obtain one at https://mozilla.org/mpl/2.0/.
 */

use super::{
	action::Action,
	named::{JobName, TaskName},
	read_filter::Kind as ReadFilterKind,
};
use fetcher_core::{
	auth as c_auth, read_filter::ReadFilter as CReadFilter, task::entry_to_msg_map::EntryToMsgMap,
	utils::DisplayDebug,
};

use std::{
	error::Error as StdError,
	fmt::{Debug, Display},
	io,
	path::Path,
};

pub enum ExternalDataResult<T, E = ExternalDataError> {
	Ok(T),
	Unavailable,
	Err(E),
}

pub trait ProvideExternalData {
	type ReadFilter: CReadFilter + 'static;

	fn twitter_token(&self) -> ExternalDataResult<(String, String)> {
		ExternalDataResult::Unavailable
	}

	fn google_oauth2(&self) -> ExternalDataResult<c_auth::Google> {
		ExternalDataResult::Unavailable
	}
	fn email_password(&self) -> ExternalDataResult<String> {
		ExternalDataResult::Unavailable
	}
	fn telegram_bot_token(&self) -> ExternalDataResult<String> {
		ExternalDataResult::Unavailable
	}
	fn discord_bot_token(&self) -> ExternalDataResult<String> {
		ExternalDataResult::Unavailable
	}

	fn read_filter(
		&self,
		_job: &JobName,
		_task: Option<&TaskName>,
		_expected_rf: ReadFilterKind,
	) -> ExternalDataResult<Self::ReadFilter> {
		ExternalDataResult::Unavailable
	}

	fn entry_to_msg_map(
		&self,
		_job: &JobName,
		_task: Option<&TaskName>,
	) -> ExternalDataResult<EntryToMsgMap> {
		ExternalDataResult::Unavailable
	}

	/// import action `name`
	fn import(&self, _name: &str) -> ExternalDataResult<Vec<Action>> {
		ExternalDataResult::Unavailable
	}
}

#[derive(thiserror::Error, Debug)]
pub enum ExternalDataError {
	#[error("IO error{}{}", .payload.is_some().then_some(": ").unwrap_or_default(), if let Some(p) = payload.as_ref() { p as &dyn Display } else { &"" })]
	Io {
		source: io::Error,
		payload: Option<Box<dyn DisplayDebug + Send + Sync>>,
	},

	#[error("Incompatible read filter types: in config: \"{expected}\" and found: \"{found}\"{}{}", .payload.is_some().then_some(": ").unwrap_or_default(), if let Some(p) = payload.as_ref() { p as &dyn Display } else { &"" })]
	ReadFilterIncompatibleTypes {
		expected: ReadFilterKind,
		found: ReadFilterKind,
		payload: Option<Box<dyn DisplayDebug + Send + Sync>>,
	},

	#[error("Action \"{}\" not found", .0)]
	ActionNotFound(String),

	#[error("Can't parse action \"{name}\": {err}")]
	ActionParsingError {
		name: String,
		err: Box<dyn StdError + Send + Sync>,
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
			payload: Some(Box::new(format!("path: {}", path.display()))),
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
