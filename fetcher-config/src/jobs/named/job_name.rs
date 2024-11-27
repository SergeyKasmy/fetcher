/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{
	borrow::Borrow,
	fmt::{self, Display},
	ops::Deref,
	path::Path,
	sync::Arc,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JobName(pub Arc<str>);

impl JobName {
	/// Converts a path to a job config to the name of that job.
	///
	/// # Example
	/// ```
	/// # use std::path::Path;
	/// # use fetcher_config::jobs::named::JobName;;
	/// let config_path = Path::new("/etc/fetcher/jobs/foo/bar.yml");
	/// let config_directory = Path::new("/etc/fetcher/jobs");
	///
	/// let job_name = JobName::from_job_config_path(config_path, config_directory);
	/// assert_eq!(job_name.as_str(), "foo/bar");
	/// ```
	///
	/// # Panics
	/// if the job config path isn't located inside job config directory
	#[must_use]
	pub fn from_job_config_path(job_config_path: &Path, job_config_directory: &Path) -> Self {
		job_config_path
			.strip_prefix(job_config_directory)
			.expect("prefix should always be present because we just appended it")
			.with_extension("")
			.to_string_lossy()
			.into()
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		self
	}
}

impl<T: Into<Arc<str>>> From<T> for JobName {
	fn from(value: T) -> Self {
		Self(value.into())
	}
}

impl Deref for JobName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Borrow<str> for JobName {
	fn borrow(&self) -> &str {
		self
	}
}

impl Display for JobName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\"{}\"", self.0)
	}
}
