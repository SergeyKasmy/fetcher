use std::{any::Any, fmt};

use crate::error::FetcherError;

pub enum JobResult {
	/// The job successfully and no tasks returned Err
	Ok,

	/// One or more task returned errors
	Err(Vec<FetcherError>),

	/// The job panicked
	Panicked {
		payload: Box<dyn Any + Send + 'static>,
	},
}

impl JobResult {
	pub fn unwrap(self) {
		match self {
			Self::Ok => (),
			Self::Err(errors) => {
				unwrap_failed("called `JobResult::unwrap()` on an `Err` value", &errors);
			}
			Self::Panicked { payload } => unwrap_failed(
				"called `JobResult::unwrap()` on a `Panicked` value",
				&payload,
			),
		}
	}

	pub fn expect(self, msg: &str) {
		match self {
			Self::Ok => (),
			Self::Err(errors) => unwrap_failed(msg, &errors),
			Self::Panicked { payload } => unwrap_failed(msg, &payload),
		}
	}
}

fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) {
	panic!("{msg}: {error:?}");
}
