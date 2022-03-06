// enum Id {
// 	Id(String),
// 	Date(String),
// }

// enum ReadFilterKind {
// 	Newer(Id),
// 	NotPresentInReadList(Vec<Id>),
// }

// pub(crate) struct ReadFilter {
// 	pub(crate) service_name: String,
// 	pub(crate) kind: ReadFilterKind,
// }

// impl ReadFilter {
// 	pub(crate) fn remove_read(&self, ) -> bool {
// 		match self.read {
// 			ReadKind::Id(id) => ,
// 			ReadKind::Date(date) => todo!(),
// 			ReadKind::List(list) => todo!(),
// 		}
// 	}
// }

pub trait Id {
	fn id(&self) -> &str;
}

#[derive(Debug)]
pub struct ReadFilterNewer {
	/* FIXME: temporary pub */ pub(crate) last_read_id: Option<String>,
}

impl ReadFilterNewer {
	pub(crate) fn new(last_read_id: Option<String>) -> Self {
		Self { last_read_id }
	}

	pub(crate) fn set_last_read_id(&mut self, last_read_id: String) {
		self.last_read_id = Some(last_read_id);
	}

	/// Make sure list is sorted newest to oldest
	pub(crate) fn remove_read_from<T: Id>(&self, list: &mut Vec<T>) {
		if let Some(last_read_id) = &self.last_read_id {
			if let Some(last_read_id_pos) = list.iter().position(|x| x.id() == last_read_id) {
				list.drain(last_read_id_pos..);
			}
		}
	}

	/// Check if current_id is unread
	/// Make sure id_list is sorted newest to oldest
	pub(crate) fn is_unread(&self, current_id: &str, id_list: &[&str]) -> bool {
		if let Some(last_read_id) = &self.last_read_id {
			if current_id == last_read_id {
				return false;
			}
			// None => Nether current id nor last read id is first
			// Some(true) => current id is is_unread
			// Some(false) => current id is read
			return id_list
				.iter()
				.fold(None, |acc, &x| match acc {
					None => {
						if x == current_id {
							Some(true)
						} else if x == last_read_id {
							Some(false)
						} else {
							None
						}
					}
					some => some,
				})
				.expect("current_id not found in id_list");
		}

		true
	}
}
