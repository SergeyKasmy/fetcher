use serde::{Deserialize, Serialize};

use crate::read_filter;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReadFilterKind {
	NewerThanRead,
	NotPresentInReadList,
}

impl ReadFilterKind {
	pub(crate) fn parse(self) -> read_filter::ReadFilterKind {
		match self {
			ReadFilterKind::NewerThanRead => read_filter::ReadFilterKind::NewerThanLastRead,
			ReadFilterKind::NotPresentInReadList => {
				read_filter::ReadFilterKind::NotPresentInReadList
			}
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ReadFilter {
	NewerThanRead(ReadFilterNewer),
	NotPresentInReadList(ReadFilterNotPresent),
}

impl ReadFilter {
	pub(crate) fn parse(self) -> read_filter::ReadFilter {
		match self {
			ReadFilter::NewerThanRead(x) => read_filter::ReadFilter::NewerThanLastRead(x.parse()),
			ReadFilter::NotPresentInReadList(x) => {
				read_filter::ReadFilter::NotPresentInReadList(x.parse())
			}
		}
	}

	pub(crate) fn unparse(read_filter: read_filter::ReadFilter) -> Option<Self> {
		Some(match read_filter {
			read_filter::ReadFilter::NewerThanLastRead(x) => {
				ReadFilter::NewerThanRead(ReadFilterNewer::unparse(x)?)
			}
			read_filter::ReadFilter::NotPresentInReadList(x) => {
				ReadFilter::NotPresentInReadList(ReadFilterNotPresent::unparse(x)?)
			}
		})
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ReadFilterNewer {
	last_read_id: String,
}

impl ReadFilterNewer {
	pub(crate) fn parse(self) -> read_filter::newer::ReadFilterNewer {
		read_filter::newer::ReadFilterNewer {
			last_read_id: Some(self.last_read_id),
		}
	}

	pub(crate) fn unparse(read_filter: read_filter::newer::ReadFilterNewer) -> Option<Self> {
		read_filter
			.last_read_id
			.map(|last_read_id| Self { last_read_id })
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ReadFilterNotPresent {
	read_list: Vec<String>,
}

impl ReadFilterNotPresent {
	pub(crate) fn parse(self) -> read_filter::not_present::ReadFilterNotPresent {
		read_filter::not_present::ReadFilterNotPresent {
			read_list: self.read_list.into(),
		}
	}

	pub(crate) fn unparse(
		read_filter: read_filter::not_present::ReadFilterNotPresent,
	) -> Option<Self> {
		if !read_filter.read_list.is_empty() {
			Some(Self {
				read_list: read_filter.read_list.into(),
			})
		} else {
			None
		}
	}
}
