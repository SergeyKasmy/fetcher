/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`CancellationToken`] type

use tokio::sync::watch::{self, channel};

/// The receiving end of a channel that is notified when a job should be cancelled
#[derive(Clone, Debug)]
pub struct CancellationToken(pub(crate) watch::Receiver<()>);

impl CancellationToken {
	/// Creates a new [`CancellationToken`] and returns the sending part of the underlying channel back
	#[must_use]
	pub fn new() -> (Self, watch::Sender<()>) {
		let (tx, rx) = channel(());
		(Self(rx), tx)
	}

	/// Blocks the current task until the sender calls asks us to stop
	pub async fn wait(&mut self) {
		// assume closed channel = cancelled
		_ = self.0.changed().await;
	}

	/// Checks if the [`CancellationToken`] has been signaled to stop without blocking the calling thread
	#[must_use]
	pub fn is_cancelled(&self) -> bool {
		// assume closed channel = cancelled
		self.0.has_changed().unwrap_or(true)
	}
}

/// Returns when the [`CancellationToken`] signals that the job should be stopped.
/// If `token` is `None`, then blocks forever
pub(crate) async fn cancel_wait(token: Option<&mut CancellationToken>) {
	match token {
		Some(token) => token.wait().await,
		None => std::future::pending().await,
	}
}
