use std::env;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Result;
use news_reader::providers::email::EmailFilter;
use news_reader::providers::Email;
use news_reader::providers::Rss;
use news_reader::providers::Twitter;
use news_reader::telegram::Telegram;
use news_reader::Config;
use teloxide::Bot;
use toml::Value;

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let conf = include_str!("config.toml");
	let parsed = parse_conf(conf).await?;

	Ok(news_reader::run(parsed).await?)
}

async fn parse_conf(c: &str) -> Result<Vec<Config>> {
	let t = Value::from_str(c)?;
	let t = Box::new(t);
	let t = Box::leak(t);

	let bot = Bot::new(env::var("BOT_TOKEN").unwrap());

	let mut confs: Vec<Config> = Vec::new();
	for x in t.as_table().unwrap() {
		let sink = Telegram::new(
			bot.clone(),
			env::var(format!("{}_CHAT_ID", x.0.to_ascii_uppercase())).unwrap(),
		);
		let table = x.1.as_table().unwrap();
		let conf = match table["type"].as_str().unwrap() {
			"rss" => {
				let source =
					Rss::new(x.0.to_string(), table["url"].as_str().unwrap().to_string()).into();

				Config { source, sink }
			}
			"twitter" => {
				let filter = table["filter"]
					.as_array()
					.unwrap()
					.iter()
					.map(|x| x.as_str().unwrap().to_string())
					.collect::<Vec<String>>();

				let source = Twitter::new(
					x.0.to_string(),
					table["pretty_name"].as_str().unwrap().to_string(),
					table["handle"].as_str().unwrap().to_string(),
					env::var("TWITTER_API_KEY").unwrap(),
					env::var("TWITTER_API_KEY_SECRET").unwrap(),
					filter,
				)
				.await
				.unwrap()
				.into();

				Config { source, sink }
			}
			"email" => {
				let filter = {
					let filter_table = table["filter"].as_table().unwrap();

					let sender = filter_table
						.get("sender")
						// TODO: move out to a separate local(?) fn
						.map(|x| x.as_str().unwrap().to_string());
					let subject = filter_table.get("subject").map(|x| {
						x.as_array()
							.unwrap()
							.iter()
							.map(|x| x.as_str().unwrap().to_string())
							.collect::<Vec<_>>()
					});

					EmailFilter { sender, subject }
				};

				let source = Email::new(
					x.0.to_string(),
					table["imap"].as_str().unwrap().to_string(),
					env::var("EMAIL").unwrap(),
					env::var("EMAIL_PASS").unwrap(),
					filter,
					table["remove"].as_bool().unwrap(),
					Some(table["footer"].as_str().unwrap().to_string()),
				)
				.into();

				Config { source, sink }
			}
			_ => return Err(anyhow!("Not a valid type")),
		};

		confs.push(conf);
	}

	Ok(confs)
}
