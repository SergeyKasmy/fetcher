/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::StaticStr;

/// A list of filters passed to the IMAP server
#[derive(bon::Builder, Debug)]
pub struct Filters {
	/// Get emails only containing these strings in the subject
	#[builder(field)]
	pub subjects: Option<Vec<StaticStr>>,

	/// Get all emails matching all above criteria but not containing any of these strings in the subject
	#[builder(field)]
	pub exclude_subjects: Option<Vec<StaticStr>>,

	/// Get emails only from this sender
	#[builder(into)]
	pub sender: Option<StaticStr>,
}

impl<S: filters_builder::State> FiltersBuilder<S> {
	pub fn subject(mut self, value: impl Into<StaticStr>) -> Self {
		self.subjects.get_or_insert_default().push(value.into());
		self
	}

	pub fn exclude_subject(mut self, value: impl Into<StaticStr>) -> Self {
		self.exclude_subjects
			.get_or_insert_default()
			.push(value.into());
		self
	}
}
