/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Build script - fetch git branch and commit hash

use std::error::Error;

use vergen_git2::{Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn Error>> {
	//EmitBuilder::builder().git_sha(true).git_branch().emit()?;

	Emitter::default()
		.add_instructions(&Git2Builder::default().sha(true).branch(true).build()?)?
		.emit()?;

	Ok(())
}
