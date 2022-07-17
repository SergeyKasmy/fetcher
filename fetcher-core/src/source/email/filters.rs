/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[derive(Debug)]
pub struct Filters {
	pub sender: Option<String>,
	pub subjects: Option<Vec<String>>,
	pub exclude_subjects: Option<Vec<String>>,
}
