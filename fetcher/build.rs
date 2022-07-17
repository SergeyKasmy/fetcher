/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use vergen::{vergen, Config, SemverKind, ShaKind};

fn main() {
	let mut conf = Config::default();
	*conf.git_mut().sha_kind_mut() = ShaKind::Short;
	*conf.git_mut().semver_kind_mut() = SemverKind::Lightweight;
	*conf.git_mut().semver_dirty_mut() = Some("-dirty");
	vergen(conf).unwrap();
}
