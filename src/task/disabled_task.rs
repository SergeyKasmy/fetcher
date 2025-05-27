/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`DisabledTask`] type

use crate::{error::FetcherError, maybe_send::MaybeSendSync, task::OpaqueTask};

/// A wrapper type that disables the execution of a task.
///
/// This type wraps a task and implements [`OpaqueTask`] in a way that
/// makes [`OpaqueTask::run`] a no-op.
///
/// See [`OpaqueTask::disable`].
pub struct DisabledTask<T>(pub T);

impl<T> DisabledTask<T> {
	/// Gets the wrapped task out of [`DisabledTask`].
	///
	/// Pattern matching can be used as well.
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T: MaybeSendSync> OpaqueTask for DisabledTask<T> {
	async fn run(&mut self) -> Result<(), FetcherError> {
		Ok(())
	}

	fn set_ctrlc_channel(&mut self, _channel: crate::ctrl_c_signal::CtrlCSignalChannel) {}
}
