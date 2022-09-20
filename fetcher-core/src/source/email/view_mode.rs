/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// A view mode for the IMAP connection
#[derive(Debug)]
pub enum ViewMode {
	/// Completely read only, never modifies anything
	ReadOnly,
	/// Mark the read ones as read but retain them in the inbox
	MarkAsRead,
	/// Delete the read ones
	/// In Gmail this normally marks them as archived, unless changed in the settings
	Delete,
}
