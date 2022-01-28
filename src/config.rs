// FIXME: use lib error type
use anyhow::Result;
use std::str::FromStr;
use teloxide::Bot;
use toml::Value;

use crate::{
	providers::{email::EmailFilter, Email, Provider, Rss, Twitter},
	telegram::Telegram,
};

type Sink = Telegram;

#[derive(Debug)]
pub struct Config {
	pub name: String,
	pub source: Provider,
	pub sink: Sink,
}

/*
impl FromStr for Vec<Config> {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {

	}
}
*/

fn env(s: &str) -> String {
	std::env::var(s).unwrap_or_else(|e| panic!("{} env not found: {}", s, e))
}

impl Config {
	pub async fn parse(conf_raw: &str) -> Result<Vec<Self>> {
		let tbl = Value::from_str(conf_raw)?;
		let bot = Bot::new(env("BOT_TOKEN"));

		let mut confs: Vec<Self> = Vec::new();
		// NOTE: should be safe. AFAIK the root of a TOML is always a table
		for entry in tbl.as_table().unwrap() {
			let chat_id = format!("{}_CHAT_ID", entry.0.to_ascii_uppercase());
			let name = entry.0.clone();
			let sink = Telegram::new(bot.clone(), env(&chat_id));
			let conf = match entry
				.1
				.as_table()
				.unwrap_or_else(|| panic!("{} does not contain a table", entry.0))
				.get("type")
				.unwrap_or_else(|| panic!("{} doesn't contain type field", entry.0))
				.as_str()
				.unwrap_or_else(|| panic!("{}'s type field is not a valid string", entry.0))
			{
				"rss" => Self::parse_rss(entry, name, sink),
				"twitter" => Self::parse_twitter(entry, name, sink).await,
				"email" => Self::parse_email(entry, name, sink),
				t => panic!("{} is not a valid type for {}", t, entry.0),
			};

			confs.push(conf);
		}

		Ok(confs)
	}

	fn parse_rss(c: (&String, &Value), name: String, sink: Sink) -> Self {
		let table =
			c.1.as_table()
				.unwrap_or_else(|| panic!("{} does not contain a table", c.0));
		let source = Rss::new(
			c.0.to_string(),
			table
				.get("url")
				.unwrap_or_else(|| panic!("{} doesn't contain url field", c.0))
				.as_str()
				.unwrap_or_else(|| panic!("{}'s url field is not a valid string", c.0))
				.to_string(),
		)
		.into();

		Self { name, source, sink }
	}

	async fn parse_twitter(c: (&String, &Value), name: String, sink: Sink) -> Self {
		let table =
			c.1.as_table()
				.unwrap_or_else(|| panic!("{} does not contain a table", c.0));
		let filter = table
			.get("filter")
			.unwrap_or_else(|| panic!("{} doesn't contain filter field", c.0))
			.as_array()
			.unwrap_or_else(|| panic!("{}'s filter is not an array", c.0))
			.iter()
			.map(|x| {
				x.as_str()
					.unwrap_or_else(|| panic!("{}'s filter is not a valid string", c.0))
					.to_string()
			})
			.collect::<Vec<String>>();

		let source = Twitter::new(
			c.0.to_string(),
			table
				.get("pretty_name")
				.unwrap_or_else(|| panic!("{} doesn't contain pretty_name field", c.0))
				.as_str()
				.unwrap_or_else(|| panic!("{}' pretty_name is not a valid string", c.0))
				.to_string(),
			table
				.get("handle")
				.unwrap_or_else(|| panic!("{} doesn't contain handle field", c.0))
				.as_str()
				.unwrap_or_else(|| panic!("{}'s handle is not a valid string", c.0))
				.to_string(),
			env("TWITTER_API_KEY"),
			env("TWITTER_API_KEY_SECRET"),
			filter,
		)
		.await
		.unwrap() // FIXME: use proper errors
		.into();

		Self { name, source, sink }
	}

	fn parse_email(c: (&String, &Value), name: String, sink: Sink) -> Self {
		let table =
			c.1.as_table()
				.unwrap_or_else(|| panic!("{} does not contain a table", c.0));
		let filter = {
			let filter_table = table
				.get("filter")
				.unwrap_or_else(|| panic!("{} doesn't contain filter field", c.0))
				.as_table()
				.unwrap_or_else(|| panic!("{}'s filter is not a valid table", c.0));

			let sender = filter_table
				.get("sender")
				// TODO: move out to a separate local(?) fn
				.map(|x| {
					x.as_str()
						.unwrap_or_else(|| {
							panic!("{}'s filter sender field is not a valid string", c.0)
						})
						.to_string()
				});
			let subject = filter_table.get("subject").map(|x| {
				x.as_array()
					.unwrap_or_else(|| panic!("{}'s filter subject is not an valid array", c.0))
					.iter()
					.map(|x| {
						x.as_str()
							.unwrap_or_else(|| {
								panic!("{}'s filter subject is not a valid string", c.0)
							})
							.to_string()
					})
					.collect::<Vec<_>>()
			});

			EmailFilter { sender, subject }
		};

		let source = Email::new(
			c.0.to_string(),
			table
				.get("imap")
				.unwrap_or_else(|| panic!("{} doesn't contain imap field", c.0))
				.as_str()
				.unwrap_or_else(|| panic!("{}'s imap is not a valid string", c.0))
				.to_string(),
			env("EMAIL"),
			env("EMAIL_PASS"),
			filter,
			table
				.get("remove")
				.unwrap_or_else(|| panic!("{} doesn't contain remove field", c.0))
				.as_bool()
				.unwrap_or_else(|| panic!("{}'s remove is not a valid bool", c.0)),
			Some(
				table
					.get("footer")
					.unwrap_or_else(|| panic!("{} doesn't contain footer field", c.0))
					.as_str()
					.unwrap_or_else(|| panic!("{}'s footer is not a valid string", c.0))
					.to_string(),
			),
		)
		.into();

		Self { name, source, sink }
	}
}
