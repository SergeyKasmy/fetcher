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
	/// Blocks the current task until a Ctrl-C signal has been received
	#[expect(clippy::missing_panics_doc, reason = "never actually panics")]
	pub async fn signaled(&mut self) {
		self.0
			.changed()
			.await
			.expect("Sender should be running in a detached tokio task and never dropped");
	}
}

/// Returns when the CtrlC channel signals that Ctrl-C has been pressed.
/// If ctrlc_chan is None, then it never returns
pub(crate) async fn ctrlc_signaled(ctrlc_chan: Option<&mut CtrlCSignalChannel>) {
	match ctrlc_chan {
		Some(ctrlc_chan) => ctrlc_chan.signaled().await,
		None => std::future::pending().await,
	}
}
