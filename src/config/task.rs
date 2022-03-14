/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{read_filter, sink::Sink, source, source::Source};
use crate::{
	error::{Error, Result},
	task,
};

// #[derive(Deserialize, Debug)]
// #[serde(transparent, rename = "templates")]
// pub struct Templates(pub Option<Vec<PathBuf>>);

#[derive(Deserialize, Debug)]
pub struct Templates {
	pub templates: Option<Vec<PathBuf>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
	disabled: Option<bool>,
	#[serde(rename = "read_filter_type")]
	read_filter_kind: read_filter::Kind,
	refresh: u64,
	source: Source,
	sink: Sink,
}

impl Task {
	pub fn parse(self, conf_path: &Path) -> Result<task::Task> {
		if let read_filter::Kind::NewerThanRead = self.read_filter_kind {
			if let Source::Html(html) = &self.source {
				if let source::html::query::IdQueryKind::Date = html.idq.kind {
					return Err(Error::IncompatibleConfigValues(
						r#"HTML source id of type "date" isn't compatible with read_filter_type of "not_present_in_read_list""#,
						conf_path.to_owned(),
					));
				}
			}
		}
		Ok(task::Task {
			disabled: self.disabled.unwrap_or(false),
			read_filter_kind: self.read_filter_kind.parse(),
			refresh: self.refresh,
			sink: self.sink.parse()?,
			// sink: todo!(),
			source: self.source.parse()?,
		})
	}
}

// #[cfg(test)]
// mod tests {
// 	use super::*;

// 	#[test]
// 	fn conf() {
// 		let s = std::fs::read_to_string("debug_data/cfg/tasks/csgo-updates.yaml").unwrap();
// 		let _task: Task = serde_yaml::from_str(&s).unwrap();
// 	}

// #[test]
// fn ser() {
// 	use crate::config::source::html::query::*;
// 	use crate::config::source::html::Html;
// 	use std::str::FromStr;

// 	let source = Source::Html(Html {
// 		url: url::Url::from_str("https://blog.counter-strike.net/index.php/category/updates/")
// 			.unwrap(),
// 		itemq: vec![Query {
// 			kind: QueryKind::Attr {
// 				name: "id".to_owned(),
// 				value: "post_container".to_owned(),
// 			},
// 			ignore: None,
// 		}],
// 		textq: vec![TextQuery {
// 			prepend: None,
// 			inner: QueryData {
// 				data_location: DataLocation::Text,
// 				query: vec![Query {
// 					kind: QueryKind::Tag {
// 						value: "p".to_owned(),
// 					},
// 					ignore: Some(QueryKind::Class {
// 						value: "post_date".to_owned(),
// 					}),
// 				}],
// 			},
// 		}],
// 		idq: IdQuery {
// 			kind: IdQueryKind::String,
// 			inner: QueryData {
// 				data_location: DataLocation::Attr {
// 					value: "href".to_owned(),
// 				},
// 				query: vec![
// 					Query {
// 						kind: QueryKind::Tag {
// 							value: "h2".to_owned(),
// 						},
// 						ignore: None,
// 					},
// 					Query {
// 						kind: QueryKind::Tag {
// 							value: "a".to_owned(),
// 						},
// 						ignore: None,
// 					},
// 				],
// 			},
// 		},
// 		linkq: LinkQuery {
// 			prepend: None,
// 			inner: QueryData {
// 				data_location: DataLocation::Attr {
// 					value: "href".to_owned(),
// 				},
// 				query: vec![
// 					Query {
// 						kind: QueryKind::Tag {
// 							value: "h2".to_owned(),
// 						},
// 						ignore: None,
// 					},
// 					Query {
// 						kind: QueryKind::Tag {
// 							value: "a".to_owned(),
// 						},
// 						ignore: None,
// 					},
// 				],
// 			},
// 		},
// 		imgq: None,
// 	});

// 	let task = Task {
// 		disabled: Some(true),
// 		read_filter_kind: read_filter::Kind::NewerThanRead,
// 		refresh: 1,
// 		source,
// 	};

// 	let s = serde_yaml::to_string(&task).unwrap();
// 	std::fs::write("/tmp/csgo-updates.yaml", s).unwrap();
// }
// }
