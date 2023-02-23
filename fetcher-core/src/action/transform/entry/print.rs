/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains a debug print transform-like function [`print()`]

use async_trait::async_trait;

use crate::{
	action::transform::result::TransformedEntry,
	entry::Entry,
	sink::{Sink, Stdout},
};

use std::{convert::Infallible, fmt::Write as _};

use super::TransformEntry;

#[derive(Debug)]
pub struct DebugPrint;

#[async_trait]
impl TransformEntry for DebugPrint {
	type Err = Infallible;

	async fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let mut msg = entry.msg.clone();

		// append id and raw_contents entry fields to the body to help in debugging
		msg.body = {
			let mut body = msg.body.unwrap_or_else(|| "None".to_owned());
			let _ = write!(
				body,
				"\n\nid: {:?}\n\nraw_contents: {:?}",
				entry.id, entry.raw_contents
			);
			Some(body)
		};

		Stdout
			.send(msg, Some("print transform"))
			.await
			.expect("stdout is unavailable");

		Ok(Vec::new())
	}
}
