/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{convert::Infallible, time::Duration};

use super::{ContinueJob, Trigger, sleep};

/// Re-trigger the job every time after a time period has passed
#[derive(Clone, Copy, Debug)]
pub struct Every(pub Duration);

impl Trigger for Every {
	type Err = Infallible;

	async fn wait(&mut self) -> Result<ContinueJob, Self::Err> {
		sleep(self.0).await;
		Ok(ContinueJob::Yes)
	}

	fn twice_as_duration(&self) -> Duration {
		self.0 * 2
	}
}
