use std::env::var;
use std::str::FromStr;
use teloxide::Bot;
use toml::{value::Map, Value};

use crate::{
	error::Error,
	error::Result,
	sink::{Sink, Telegram},
	source::{email::EmailFilters, Email, Rss, Source, Twitter},
};

fn env(s: &str) -> Result<String> {
	var(s).map_err(|_| Error::GetEnvVar(s.to_string()))
}

#[derive(Debug)]
pub struct Config {
	pub name: String,
	pub source: Source,
	pub sink: Sink,
	pub refresh: u64,
}

impl Config {
	pub async fn parse(conf_raw: &str) -> Result<Vec<Self>> {
		let tbl = Value::from_str(conf_raw)?;
		let bot = Bot::new(env("BOT_TOKEN")?);

		let mut confs: Vec<Self> = Vec::new();
		// NOTE: unwrapping should be safe. AFAIK the root of a TOML is always a table
		for (name, table) in tbl.as_table().unwrap() {
			let table = table.as_table().ok_or(Error::ConfigMissingField {
				name: name.clone(),
				field: "table",
			})?;

			let chat_id = if !cfg!(debug_assertions) {
				format!("{}_CHAT_ID", name.to_ascii_uppercase())
			} else {
				"DEBUG_CHAT_ID".to_string()
			};

			let sink = Sink::Telegram(Telegram::new(bot.clone(), env(&chat_id)?));
			let source = match table
				.get("type")
				.ok_or(Error::ConfigMissingField {
					name: name.clone(),
					field: "type",
				})?
				.as_str()
				.ok_or(Error::ConfigInvalidFieldType {
					name: name.clone(),
					field: "type",
					expected_type: "string",
				})? {
				"rss" => Self::parse_rss(name, table)?,
				"twitter" => Self::parse_twitter(name, table).await?,
				"email" => Self::parse_email(name, table)?,
				t => panic!("{t} is not a valid type for {name}"),
			};
			let refresh = table
				.get("refresh")
				.ok_or(Error::ConfigMissingField {
					name: name.clone(),
					field: "refresh",
				})?
				.as_integer()
				.ok_or(Error::ConfigInvalidFieldType {
					name: name.clone(),
					field: "refresh",
					expected_type: "integer",
				})? as u64; // FIXME: figure out if casting with as can cause problems

			confs.push(Config {
				name: name.clone(),
				source,
				sink,
				refresh,
			});
		}

		Ok(confs)
	}

	fn parse_rss(name: &str, table: &Map<String, Value>) -> Result<Source> {
		Ok(Rss::new(
			name.to_string(),
			table
				.get("url")
				.ok_or(Error::ConfigMissingField {
					name: name.to_string(),
					field: "url",
				})?
				.as_str()
				.ok_or(Error::ConfigInvalidFieldType {
					name: name.to_string(),
					field: "url",
					expected_type: "string",
				})?
				.to_string(),
		)
		.into())
	}

	async fn parse_twitter(name: &str, table: &Map<String, Value>) -> Result<Source> {
		let filter = table
			.get("filter")
			.ok_or(Error::ConfigMissingField {
				name: name.to_string(),
				field: "filter",
			})?
			.as_array()
			.ok_or(Error::ConfigInvalidFieldType {
				name: name.to_string(),
				field: "filter",
				expected_type: "array",
			})?
			.iter()
			.map(|x| {
				Ok(x.as_str()
					.ok_or(Error::ConfigInvalidFieldType {
						name: name.to_string(),
						field: "filter",
						expected_type: "string",
					})?
					.to_string())
			})
			.collect::<Result<Vec<String>>>()?;

		Ok(Twitter::new(
			name.to_string(),
			table
				.get("pretty_name")
				.ok_or(Error::ConfigMissingField {
					name: name.to_string(),
					field: "pretty_name",
				})?
				.as_str()
				.ok_or(Error::ConfigInvalidFieldType {
					name: name.to_string(),
					field: "pretty_name",
					expected_type: "string",
				})?
				.to_string(),
			table
				.get("handle")
				.ok_or(Error::ConfigMissingField {
					name: name.to_string(),
					field: "handle",
				})?
				.as_str()
				.ok_or(Error::ConfigInvalidFieldType {
					name: name.to_string(),
					field: "handle",
					expected_type: "string",
				})?
				.to_string(),
			env("TWITTER_API_KEY")?,
			env("TWITTER_API_KEY_SECRET")?,
			filter,
		)
		.await?
		.into())
	}

	fn parse_email(name: &str, table: &Map<String, Value>) -> Result<Source> {
		let filters = {
			let filters_table = table
				.get("filters")
				.ok_or(Error::ConfigMissingField {
					name: name.to_string(),
					field: "filters",
				})?
				.as_table()
				.ok_or(Error::ConfigInvalidFieldType {
					name: name.to_string(),
					field: "filters",
					expected_type: "table",
				})?;

			let sender = filters_table
				.get("sender")
				.map(|x| {
					x.as_str()
						.ok_or(Error::ConfigInvalidFieldType {
							name: name.to_string(),
							field: "filters sender",
							expected_type: "string",
						})
						.map(ToString::to_string)
				})
				.transpose()?;

			let subjects = filters_table
				.get("subjects")
				.map(|a| {
					a.as_array()
						.ok_or(Error::ConfigMissingField {
							name: name.to_string(),
							field: "filters subject",
						})?
						.iter()
						.map(|s| {
							s.as_str()
								.ok_or(Error::ConfigInvalidFieldType {
									name: name.to_string(),
									field: "filters subjects",
									expected_type: "string",
								})
								.map(ToString::to_string)
						})
						.collect::<Result<Vec<_>>>()
				})
				.transpose()?;

			let exclude_subjects = filters_table
				.get("exclude_subjects")
				.map(|a| {
					a.as_array()
						.ok_or(Error::ConfigMissingField {
							name: name.to_string(),
							field: "filters exclude_subjects",
						})?
						.iter()
						.map(|s| {
							s.as_str()
								.ok_or(Error::ConfigInvalidFieldType {
									name: name.to_string(),
									field: "filters exclude_subjects",
									expected_type: "string",
								})
								.map(ToString::to_string)
						})
						.collect::<Result<Vec<_>>>()
				})
				.transpose()?;

			EmailFilters {
				sender,
				subjects,
				exclude_subjects,
			}
		};

		let imap = table
			.get("imap")
			.ok_or(Error::ConfigMissingField {
				name: name.to_string(),
				field: "imap",
			})?
			.as_str()
			.ok_or(Error::ConfigInvalidFieldType {
				name: name.to_string(),
				field: "imap",
				expected_type: "string",
			})?
			.to_string();

		let email = table
			.get("email")
			.ok_or(Error::ConfigMissingField {
				name: name.to_string(),
				field: "email",
			})?
			.as_str()
			.ok_or(Error::ConfigInvalidFieldType {
				name: name.to_string(),
				field: "email",
				expected_type: "string",
			})?
			.to_string();

		let remove = table
			.get("remove")
			.ok_or(Error::ConfigMissingField {
				name: name.to_string(),
				field: "remove",
			})?
			.as_bool()
			.ok_or(Error::ConfigInvalidFieldType {
				name: name.to_string(),
				field: "remove",
				expected_type: "bool",
			})?;

		let footer = Some(
			table
				.get("footer")
				.ok_or(Error::ConfigMissingField {
					name: name.to_string(),
					field: "footer",
				})?
				.as_str()
				.ok_or(Error::ConfigInvalidFieldType {
					name: name.to_string(),
					field: "footer",
					expected_type: "string",
				})?
				.to_string(),
		);

		Ok(match table
			.get("auth_type")
			.ok_or(Error::ConfigMissingField {
				name: name.to_string(),
				field: "auth_type",
			})?
			.as_str()
		{
			Some("password") => {
				let password = env("EMAIL_PASS")?;

				Email::with_password(
					name.to_string(),
					imap,
					email,
					password,
					filters,
					remove,
					footer,
				)
			}
			Some("google_oauth2") => {
				let token = env("EMAIL_GOOGLE_OAUTH2_TOKEN")?;

				Email::with_google_oauth2(
					name.to_string(),
					imap,
					email,
					token,
					filters,
					remove,
					footer,
				)
			}
			_ => {
				return Err(Error::ConfigInvalidFieldType {
					name: name.to_string(),
					field: "auth_type",
					expected_type: "string (password | oauth2)",
				});
			}
		}
		.into())
	}
}
