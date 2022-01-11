use anyhow::Result;
use news_reader::providers::email::EmailFilter;
use news_reader::providers::Email;
use news_reader::providers::Rss;
use news_reader::providers::Twitter;
use news_reader::telegram::Telegram;
use news_reader::Config;
use std::str::FromStr;
use teloxide::Bot;
use toml::Value;

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let conf_path = xdg::BaseDirectories::with_prefix("news-reader")
		.unwrap()
		.place_config_file("config.toml")
		.unwrap();
	let conf = std::fs::read_to_string(&conf_path)
		.unwrap_or_else(|_| panic!("{:?} doesn't exist", conf_path));

	let parsed = parse_conf(&conf).await?;
	Ok(news_reader::run(parsed).await?)
}

async fn parse_conf(conf_raw: &str) -> Result<Vec<Config>> {
	fn env(s: &str) -> String {
		std::env::var(s).unwrap_or_else(|e| panic!("{} env not found: {}", s, e))
	}

	let tbl = Value::from_str(conf_raw)?;
	let bot = Bot::new(env("BOT_TOKEN"));

	let mut confs: Vec<Config> = Vec::new();
	// NOTE: should be safe. AFAIK the root of a TOML is always a table
	for conf_raw in tbl.as_table().unwrap() {
		let table = conf_raw
			.1
			.as_table()
			.unwrap_or_else(|| panic!("{} does not contain a table", conf_raw.0));
		let chat_id = format!("{}_CHAT_ID", conf_raw.0.to_ascii_uppercase());
		let sink = Telegram::new(bot.clone(), env(&chat_id));
		let conf = match table
			.get("type")
			.unwrap_or_else(|| panic!("{} doesn't contain type field", conf_raw.0))
			.as_str()
			.unwrap_or_else(|| panic!("{}' type field is not a valid string", conf_raw.0))
		{
			"rss" => {
				let source = Rss::new(
					conf_raw.0.to_string(),
					table
						.get("url")
						.unwrap_or_else(|| panic!("{} doesn't contain url field", conf_raw.0))
						.as_str()
						.unwrap_or_else(|| {
							panic!("{}'s url field is not a valid string", conf_raw.0)
						})
						.to_string(),
				)
				.into();

				Config { source, sink }
			}
			"twitter" => {
				let filter = table
					.get("filter")
					.unwrap_or_else(|| panic!("{} doesn't contain filter field", conf_raw.0))
					.as_array()
					.unwrap_or_else(|| panic!("{}'s filter is not an array", conf_raw.0))
					.iter()
					.map(|x| {
						x.as_str()
							.unwrap_or_else(|| {
								panic!("{}'s filter is not a valid string", conf_raw.0)
							})
							.to_string()
					})
					.collect::<Vec<String>>();

				let source = Twitter::new(
					conf_raw.0.to_string(),
					table
						.get("pretty_name")
						.unwrap_or_else(|| {
							panic!("{} doesn't contain pretty_name field", conf_raw.0)
						})
						.as_str()
						.unwrap_or_else(|| {
							panic!("{}' pretty_name is not a valid string", conf_raw.0)
						})
						.to_string(),
					table
						.get("handle")
						.unwrap_or_else(|| panic!("{} doesn't contain handle field", conf_raw.0))
						.as_str()
						.unwrap_or_else(|| panic!("{}'s handle is not a valid string", conf_raw.0))
						.to_string(),
					env("TWITTER_API_KEY"),
					env("TWITTER_API_KEY_SECRET"),
					filter,
				)
				.await?
				.into();

				Config { source, sink }
			}
			"email" => {
				let filter = {
					let filter_table = table
						.get("filter")
						.unwrap_or_else(|| panic!("{} doesn't contain filter field", conf_raw.0))
						.as_table()
						.unwrap_or_else(|| panic!("{}'s filter is not a valid table", conf_raw.0));

					let sender = filter_table
						.get("sender")
						// TODO: move out to a separate local(?) fn
						.map(|x| {
							x.as_str()
								.unwrap_or_else(|| {
									panic!(
										"{}'s filter sender field is not a valid string",
										conf_raw.0
									)
								})
								.to_string()
						});
					let subject = filter_table.get("subject").map(|x| {
						x.as_array()
							.unwrap_or_else(|| {
								panic!("{}'s filter subject is not an valid array", conf_raw.0)
							})
							.iter()
							.map(|x| {
								x.as_str()
									.unwrap_or_else(|| {
										panic!(
											"{}'s filter subject is not a valid string",
											conf_raw.0
										)
									})
									.to_string()
							})
							.collect::<Vec<_>>()
					});

					EmailFilter { sender, subject }
				};

				let source = Email::new(
					conf_raw.0.to_string(),
					table
						.get("imap")
						.unwrap_or_else(|| panic!("{} doesn't contain imap field", conf_raw.0))
						.as_str()
						.unwrap_or_else(|| panic!("{}'s imap is not a valid string", conf_raw.0))
						.to_string(),
					env("EMAIL"),
					env("EMAIL_PASS"),
					filter,
					table
						.get("remove")
						.unwrap_or_else(|| panic!("{} doesn't contain remove field", conf_raw.0))
						.as_bool()
						.unwrap_or_else(|| panic!("{}'s remove is not a valid bool", conf_raw.0)),
					Some(
						table
							.get("footer")
							.unwrap_or_else(|| {
								panic!("{} doesn't contain footer field", conf_raw.0)
							})
							.as_str()
							.unwrap_or_else(|| {
								panic!("{}'s footer is not a valid string", conf_raw.0)
							})
							.to_string(),
					),
				)
				.into();

				Config { source, sink }
			}
			t => panic!("{} is not a valid type for {}", t, conf_raw.0),
		};

		confs.push(conf);
	}

	Ok(confs)
}
