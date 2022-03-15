use vergen::{vergen, Config, SemverKind, ShaKind};

fn main() {
	let mut conf = Config::default();
	*conf.git_mut().sha_kind_mut() = ShaKind::Short;
	*conf.git_mut().semver_kind_mut() = SemverKind::Lightweight;
	*conf.git_mut().semver_dirty_mut() = Some("-dirty");
	vergen(conf).unwrap();
}
