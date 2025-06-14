/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`DebugPrint`] transform that just prints the contents of the entry and passes it through

use std::{convert::Infallible, fmt::Write};

use super::Transform;
use crate::{
	actions::transforms::result::TransformedEntry,
	entry::Entry,
	sinks::{Sink, Stdout},
};

/// A transform that print the contents of the [`Entry`] in a debug friendly way
#[derive(Debug)]
pub struct DebugPrint;

impl Transform for DebugPrint {
	type Err = Infallible;

	async fn transform_entry(&mut self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let mut msg = entry.msg;

		// append id and raw_contents entry fields to the body to help in debugging
		msg.body = {
			let mut body = msg.body.unwrap_or_else(|| "None".to_owned());
			_ = write!(
				body,
				"\n\nid: {:?}\n\nraw_contents: {:?}",
				entry.id, entry.raw_contents
			);
			Some(body)
		};

		Stdout
			.send(&msg, None, Some("debug print"))
			.await
			.expect("stdout is unavailable");

		Ok(Vec::new())
	}
}
