/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{
	borrow::Borrow,
	fmt::{self, Display, Formatter},
};

pub trait SliceDisplayExt<'a> {
	fn display(self) -> SliceDisplay<'a>;
}

impl<'a, I, T> SliceDisplayExt<'a> for I
where
	I: Iterator<Item = &'a T>,
	T: Borrow<str> + 'a + ?Sized,
{
	fn display(self) -> SliceDisplay<'a> {
		let mut v = self.map(Borrow::borrow).collect::<Vec<&str>>();
		v.sort_unstable();
		SliceDisplay(v)
	}
}

pub struct SliceDisplay<'a>(Vec<&'a str>);

impl Display for SliceDisplay<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str("\"")?;
		f.write_str(&self.0.join("\", \""))?;
		f.write_str("\"")?;

		Ok(())
	}
}
