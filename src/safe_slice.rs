/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::borrow::Cow;

pub trait SafeSliceUntilExt {
	fn safe_slice_until(&self, index: usize) -> &Self;
	fn pretty_slice_until(&self, index: usize) -> Cow<'_, str>;
}

impl SafeSliceUntilExt for str {
	fn safe_slice_until(&self, mut index: usize) -> &Self {
		if index >= self.len() {
			return self;
		}

		while index > 0 && !self.is_char_boundary(index) {
			index -= 1;
		}

		&self[..index]
	}

	fn pretty_slice_until(&self, index: usize) -> Cow<'_, str> {
		let slice = self.safe_slice_until(index);

		if slice.len() == self.len() {
			Cow::Borrowed(slice)
		} else {
			Cow::Owned(format!("{slice}..."))
		}
	}
}
