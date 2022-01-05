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
	for x in t.as_table().ok_or(anyhow!("Not table"))? {
		let sink = Telegram::new(bot.clone(), env::var(format!("{}_CHAT_ID", x.0.to_ascii_uppercase())).unwrap());
		let table = x.1.as_table().ok_or(anyhow!("Not table"))?;
		let conf = match table["type"].as_str().ok_or(anyhow!("Not string"))? {
			"rss" => {
				let source =
					Rss::new(x.0, table["url"].as_str().ok_or(anyhow!("Not string"))?).into();

				Config { source, sink }
			}
			"twitter" => {
				let filters = Box::new(
					table["filters"]
						.as_array()
						.ok_or(anyhow!("Not array"))?
						.iter()
						.map(|x| x.as_str().ok_or(anyhow!("Not string")).unwrap())
						.collect::<Vec<&str>>(),
				);
				let filters = Box::leak(filters);

				let source = Twitter::new(
					x.0,
					table["pretty_name"].as_str().unwrap(),
					table["handle"].as_str().unwrap(),
					env::var("TWITTER_API_KEY").unwrap(),
					env::var("TWITTER_API_KEY_SECRET").unwrap(),
					Some(filters.as_slice()),
				)
				.await
				.unwrap()
				.into();

				Config { source, sink }
			}
			"email" => {
				let sender = table["filters"].as_table().unwrap()["sender"]
					.as_str()
					.unwrap();
				let mut subject = table["filters"].as_table().unwrap()["subject"]
					.as_array()
					.unwrap()
					.iter()
					.map(|x| EmailFilter::Subject(x.as_str().unwrap()))
					.collect::<Vec<EmailFilter>>();

				let mut filters = Box::new(Vec::new());
				filters.push(EmailFilter::Sender(sender));
				filters.append(&mut subject);
				let filters = Box::leak(filters);

				let source = Email::new(
					x.0,
					table["imap"].as_str().unwrap(),
					env::var("EMAIL").unwrap(),
					env::var("EMAIL_PASS").unwrap(),
					Some(filters.as_slice()),
					table["remove"].as_bool().unwrap(),
					Some(table["footer"].as_str().unwrap()),
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
