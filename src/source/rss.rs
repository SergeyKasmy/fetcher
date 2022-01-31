use crate::error::Error;
use crate::error::Result;
use crate::sink::Message;
use crate::settings::{save_last_read_id, get_last_read_id};

use rss::Channel;

#[derive(Debug)]
pub struct Rss {
	name: String,
	rss: String,
	http_client: reqwest::Client,
}

impl Rss {
	#[tracing::instrument]
	pub fn new(name: String, rss: String) -> Self {
		tracing::info!("Creatng an Rss provider");
		Self {
			name,
			rss,
			http_client: reqwest::Client::new(),
		}
	}

	#[tracing::instrument]
	pub async fn get(&mut self) -> Result<Vec<Message>> {
		let content = self
			.http_client
			.get(&self.rss)
			.send()
			.await
			.map_err(|e| Error::Fetch {
				service: format!("RSS: {}", self.name),
				why: e.to_string(),
			})?
			.bytes()
			.await
			.map_err(|e| Error::Fetch {
				service: format!("RSS: {}", self.name),
				why: e.to_string(),
			})?;
		let mut feed = Channel::read_from(&content[..]).map_err(|e| Error::Parse {
			service: format!("RSS: {}", self.name),
			why: e.to_string(),
		})?;
		tracing::debug!("Got {amount} RSS articles", amount = feed.items.len());

		let mut last_read_id = get_last_read_id(&self.name)?;
		if let Some(last_read_id_pos) = feed
			.items
			.iter()
			.position(|x| x.guid.as_ref().unwrap().value == last_read_id)
		{
			feed.items.drain(last_read_id_pos..);
		}

		let messages = feed
			.items
			.into_iter()
			.rev()
			.map(|x| {
				let text = format!(
					"<a href=\"{}\">{}</a>\n{}",
					x.link.as_deref().unwrap(),
					x.title.as_deref().unwrap(),
					x.description.as_deref().unwrap()
				); // NOTE: these fields are requred

				last_read_id = x.guid.as_ref().unwrap().value.clone(); // NOTE: crash if the feed item doesn't have a guid. That should never happen though
				Message { text, media: None }
			})
			.collect();

		save_last_read_id(&self.name, last_read_id)?;

		Ok(messages)
	}
}
