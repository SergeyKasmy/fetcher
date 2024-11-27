/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{error::Error, fmt::Display};

use color_eyre::Report;

pub trait IntoStdErrorExt {
	fn into_std_error(self) -> Box<dyn Error + Send + Sync>;
}

impl IntoStdErrorExt for Report {
	fn into_std_error(self) -> Box<dyn Error + Send + Sync> {
		Box::new(ReportStdErrorWrapper(self))
	}
}

#[derive(Debug)]
pub struct ReportStdErrorWrapper(pub Report);

impl Error for ReportStdErrorWrapper {}
impl Display for ReportStdErrorWrapper {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:#}", self.0)
	}
}
