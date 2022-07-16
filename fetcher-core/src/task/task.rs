/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;

use crate::{
	sink::Sink,
	source::{parser::Parser, Source},
};

/// Name -> Task
pub type Tasks = HashMap<String, Task>;

#[derive(Debug)]
pub struct Task {
	pub disabled: bool,
	pub refresh: u64,
	pub tag: Option<String>,
	pub source: Source,
	pub parsers: Option<Vec<Parser>>,
	pub sink: Sink,
}

// #[cfg(test)]
// mod tests {
// 	mod source_types {
// 		use teloxide::Bot;

// 		use super::super::Task;
// 		use crate::source::email::ViewMode;
// 		use crate::{
// 			sink::Sink,
// 			sink::Telegram,
// 			source::Rss,
// 			source::Source,
// 			source::{email::filters::Filters, Email},
// 		};

// 		#[test]
// 		fn one_type() {
// 			let _x = Task::new(
// 				false,
// 				1,
// 				None,
// 				Sink::Telegram(Telegram::new(Bot::new("null"), 0)),
// 				vec![Source::Rss(Rss::new("null".to_owned()))],
// 			);
// 		}

// 		#[test]
// 		fn same_types() {
// 			let _x = Task::new(
// 				false,
// 				1,
// 				None,
// 				Sink::Telegram(Telegram::new(Bot::new("null"), 0)),
// 				vec![
// 					Source::Rss(Rss::new("null".to_owned())),
// 					Source::Rss(Rss::new("null".to_owned())),
// 					Source::Rss(Rss::new("null".to_owned())),
// 				],
// 			);
// 		}

// 		#[test]
// 		#[should_panic]
// 		fn different_types() {
// 			let _x = Task::new(
// 				false,
// 				1,
// 				None,
// 				Sink::Telegram(Telegram::new(Bot::new("null"), 0)),
// 				vec![
// 					Source::Rss(Rss::new("null".to_owned())),
// 					Source::Rss(Rss::new("null".to_owned())),
// 					Source::Rss(Rss::new("null".to_owned())),
// 					Source::Rss(Rss::new("null".to_owned())),
// 					Source::Email(Email::with_password(
// 						"null".to_owned(),
// 						"null".to_owned(),
// 						"null".to_owned(),
// 						Filters {
// 							sender: None,
// 							subjects: None,
// 							exclude_subjects: None,
// 						},
// 						ViewMode::ReadOnly,
// 						None,
// 					)),
// 				],
// 			);
// 		}
// 	}
// }
