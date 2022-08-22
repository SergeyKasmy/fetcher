/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{read_filter, sink::Sink, source::Source, transform::Transform, DataSettings};
use fetcher_core::task;

#[derive(Deserialize, Debug)]
pub struct TemplatesField {
	pub templates: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
// TODO: add
// #[serde(deny_unknown_fields)]
// but allow templates templates field
// that's used elsewhere
pub struct Task {
	disabled: Option<bool>,
	#[serde(rename = "read_filter_type")]
	read_filter_kind: Option<read_filter::Kind>,
	tag: Option<String>,
	refresh: u64,
	source: Source,
	transform: Option<Vec<Transform>>,
	// TODO: several sinks
	sink: Sink,
}

impl Task {
	// TODO: return option if the disabled field was true
	pub(crate) async fn parse(
		self,
		name: &str,
		settings: &DataSettings,
	) -> Result<task::Task, crate::error::ConfigError> {
		let rf = {
			let rf = (settings.read_filter)(
				name.to_owned(),
				self.read_filter_kind.map(read_filter::Kind::parse),
			)
			.await?;
			rf.map(|rf| Arc::new(RwLock::new(rf)))
		};
		let transforms = self
			.transform
			.map(|x| {
				x.into_iter()
					.filter_map(|x| match x {
						Transform::ReadFilter => match rf.clone() {
							Some(rf) => {
								Some(Ok(fetcher_core::transform::Transform::ReadFilter(rf)))
							}
							None => {
								tracing::warn!("Can't use read filter transformer when no read filter is set up for the task!");
								None
							}
						},
						x => Some(x.parse()),
					})
					.collect::<Result<_, _>>()
			})
			.transpose()?;

		Ok(task::Task {
			disabled: self.disabled.unwrap_or(false),
			refresh: self.refresh,
			tag: self.tag.map(|s| s.replace(char::is_whitespace, "_")),
			source: self.source.parse(settings).await?,
			rf,
			transforms,
			sink: self.sink.parse(settings)?,
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
