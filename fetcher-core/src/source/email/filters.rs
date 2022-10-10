/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// A list of filters passed to the IMAP server
#[derive(Debug)]
pub struct Filters {
	/// Get emails only from this sender
	pub sender: Option<String>,
	/// Get emails only containing these strings in the subject
	pub subjects: Option<Vec<String>>,
	/// Get all emails matching all above criteria but not containing any of these strings in the subject
	pub exclude_subjects: Option<Vec<String>>,
}
