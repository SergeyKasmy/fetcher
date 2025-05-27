/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`CtrlCSignalChannel`]
// TODO: no way to create this type without scaffold

use tokio::sync::watch;

/// The receiving end of a channel that is notified when a Ctrl-C signal has been received
#[derive(Clone, Debug)]
pub struct CtrlCSignalChannel(pub(crate) watch::Receiver<()>);

impl CtrlCSignalChannel {
	/// Creates a new [`CtrlCSignalChannel`] with the provided receiving end of the watch channel
	#[must_use]
	pub fn new(recv: watch::Receiver<()>) -> Self {
		Self(recv)
	}

	/// Blocks the current task until a Ctrl-C signal has been received
	pub async fn wait(&mut self) {
		// assume closed channel = should stop
		_ = self.0.changed().await;
	}

	/// Checks if the current [`CtrlCSignalChannel`] has been signaled to stop without blocking the calling thread
	#[must_use]
	pub fn signaled(&self) -> bool {
		// assume closed channel = should stop
		self.0.has_changed().unwrap_or(true)
	}
}

/// Returns when the CtrlC channel signals that Ctrl-C has been pressed.
/// If ctrlc_chan is None, then it never returns
pub(crate) async fn ctrlc_wait(ctrlc_chan: Option<&mut CtrlCSignalChannel>) {
	match ctrlc_chan {
		Some(ctrlc_chan) => ctrlc_chan.wait().await,
		None => std::future::pending().await,
	}
}
