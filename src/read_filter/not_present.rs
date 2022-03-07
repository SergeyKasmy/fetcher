use std::collections::VecDeque;

use super::{Id, Identifiable};

#[derive(Default, Debug)]
pub struct ReadFilterNotPresent {
	pub(crate) read_list: VecDeque<String>,
}

impl ReadFilterNotPresent {
	// pub fn new(read_list: impl IntoIterator<Item = String>) -> Self {
	// 	Self {
	// 		read_list: VecDeque::from_iter(read_list),
	// 	}
	// }

	pub(crate) fn last_read(&self) -> Option<Id> {
		// TODO: why doesn't as_deref() work?
		self.read_list.back().map(|s| s.as_str())
	}

	pub(crate) fn remove_read_from<T: Identifiable>(&self, list: &mut Vec<T>) {
		list.retain(|elem| {
			!self
				.read_list
				.iter()
				.any(|read_elem_id| read_elem_id.as_str() == elem.id())
		});
	}

	pub(crate) fn mark_as_read(&mut self, id: Id) {
		self.read_list.push_back(id.to_owned());
	}
}
