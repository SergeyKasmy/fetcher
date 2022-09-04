/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{action::Action, read_filter::ReadFilter, sink::Sink, source::Source};

use std::sync::Arc;
use tokio::sync::RwLock;

/// A core primitive of [`fetcher`](`crate`).
/// Contains everything from a [`Source`] that allows to fetch some data, to a [`Sink`] that takes that data and sends it somewhere.
/// It also contains any transformators
#[derive(Debug)]
pub struct Task {
	/// An optional tag that may be put near a message body to differentiate this task from others that may be similar
	pub tag: Option<String>,
	pub rf: Option<Arc<RwLock<ReadFilter>>>,
	/// The source where to fetch some data from
	pub source: Source,
	/// A list of optional transformators which to run the data received from the source through
	pub actions: Option<Vec<Action>>,
	/// The sink where to send the data to
	pub sink: Sink,
}
