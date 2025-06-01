/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{convert::Infallible, time::Duration};

use super::{ContinueJob, Trigger};

/// Never retrigger the job, run once and stop
#[derive(Clone, Copy, Debug)]
pub struct Never;

impl Trigger for Never {
	type Err = Infallible;

	async fn wait(&mut self) -> Result<ContinueJob, Self::Err> {
		Ok(ContinueJob::No)
	}

	fn twice_as_duration(&self) -> Duration {
		Duration::ZERO
	}
}
