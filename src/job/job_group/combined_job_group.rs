/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`CombinedJobGroup`] struct

use std::pin::pin;

use futures::{Stream, StreamExt as _, stream_select};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::{JobGroup, JobId};
use crate::{job::JobResult, maybe_send::MaybeSend};

/// A job group that combines 2 other job groups and runs them concurrently to completion.
///
/// See [`JobGroup::combine_with`].
pub struct CombinedJobGroup<G1, G2>(pub G1, pub G2);

impl<G1, G2> JobGroup for CombinedJobGroup<G1, G2>
where
	G1: JobGroup,
	G2: JobGroup,
{
	fn run(self) -> impl Stream<Item = (JobId, JobResult)> + MaybeSend
	where
		Self: Sized + 'static,
	{
		let (tx, rx) = mpsc::channel(128);

		tokio::spawn(async move {
			let g1_run = pin!(self.0.run());
			let g2_run = pin!(self.1.run());

			let mut stream = stream_select!(g1_run, g2_run);
			while let Some(item) = stream.next().await {
				tx.send(item).await.unwrap();
			}
		});

		ReceiverStream::new(rx)
	}
}
