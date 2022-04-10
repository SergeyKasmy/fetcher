use serde::{Deserialize, Serialize};

use crate::source;

pub mod html;
pub mod rss;

use self::html::Html;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) enum Parser {
	Rss,
	Html(Html),
}

impl Parser {
	pub(crate) fn parse(self) -> source::parser::Parser {
		match self {
			Parser::Rss => todo!(),
			Parser::Html(x) => source::parser::Parser::Html(x.parse()),
		}
	}
}
