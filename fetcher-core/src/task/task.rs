/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;

use crate::{
	sink::Sink,
	source::{parser::Parser, Source},
};

/// A hashmap of tasks that maps Name -> Task
pub type Tasks = HashMap<String, Task>;

/// A core primitive of [`fetcher`](`crate`).
/// Contains everything from a [`Source`] that allows to fetch some data, to a [`Sink`] that takes that data and sends it somewhere.
/// It also contains any parsers
#[derive(Debug)]
pub struct Task {
	/// TODO: move these 2 out of this
	pub disabled: bool, //
	/// TODO
	pub refresh: u64,
	/// An optional tag that may be put near a message body to differentiate this task from others that may be similar
	pub tag: Option<String>,
	/// The source where to fetch some data from
	pub source: Source,
	/// A list of optional parsers which to run the data received from the source through
	pub parsers: Option<Vec<Parser>>,
	/// The sink where to send the data to
	pub sink: Sink,
}
