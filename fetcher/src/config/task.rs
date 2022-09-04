/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::action::filter::Filter;
use super::action::Action;
use super::{read_filter, sink::Sink, source::Source, TaskSettings};
use crate::error::ConfigError;
use crate::task::Task as ParsedTask;
use fetcher_core as fcore;

#[derive(Deserialize, Serialize, Debug)]
// TODO: add
// #[serde(deny_unknown_fields)]
// but allow templates templates field
// that's used elsewhere
pub struct Task {
	#[serde(rename = "read_filter_type")]
	read_filter_kind: Option<read_filter::Kind>,
	tag: Option<String>,
	refresh: u64,
	source: Source,
	#[serde(rename = "process")]
	actions: Option<Vec<Action>>,
	// TODO: several sinks
	sink: Sink,
}

impl Task {
	pub(crate) async fn parse(
		self,
		name: &str,
		settings: &TaskSettings,
	) -> Result<ParsedTask, ConfigError> {
		let rf = {
			let rf = (settings.read_filter)(
				name.to_owned(),
				self.read_filter_kind.map(read_filter::Kind::parse),
			)
			.await?;
			rf.map(|rf| Arc::new(RwLock::new(rf)))
		};
		let actions = self
			.actions
			.map(|x| {
				x.into_iter()
					.filter_map(|act| match act {
						Action::Filter(Filter::ReadFilter) => match rf.clone() {
							Some(rf) => Some(Ok(fetcher_core::action::Action::Filter(
								fetcher_core::action::filter::Kind::ReadFilter(rf),
							))),
							None => {
								tracing::warn!("Can't use read filter transformer when no read filter is set up for the task!");
								None
							}
						},
						other => Some(other.parse()),
					})
					.collect::<Result<_, _>>()
			})
			.transpose()?;

		let inner = fcore::task::Task {
			tag: self.tag.map(|s| s.replace(char::is_whitespace, "_")),
			source: self.source.parse(settings).await?,
			rf,
			actions,
			sink: self.sink.parse(settings)?,
		};

		Ok(ParsedTask {
			inner,
			refresh: self.refresh,
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
