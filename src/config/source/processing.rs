use serde::{Deserialize, Serialize};

use crate::source;

pub mod html;
pub mod rss;

use self::html::Html;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) enum Process {
	Rss,
	Html(Html),
}

impl Process {
	pub(crate) fn parse(self) -> source::processing::Process {
		match self {
			Process::Rss => todo!(),
			Process::Html(x) => source::processing::Process::Html(x.parse()),
		}
	}
}
