/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod job_display;

use itertools::Itertools;
use std::{
	cmp::Ord,
	fmt::{self, Display, Formatter},
};

pub trait SliceDisplayExt<T> {
	fn display(self) -> SliceDisplay<T>;
}

impl<I, T> SliceDisplayExt<T> for I
where
	I: Iterator<Item = T>,
	T: Display + Ord,
{
	fn display(self) -> SliceDisplay<T> {
		let mut v = self.collect::<Vec<_>>();
		v.sort_unstable();
		SliceDisplay(v)
	}
}

pub struct SliceDisplay<T>(Vec<T>);

impl<T: Display> Display for SliceDisplay<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if self.0.is_empty() {
			return Ok(());
		}

		f.write_str("\n")?;
		f.write_str(&self.0.iter().join(",\n"))?;

		Ok(())
	}
}
