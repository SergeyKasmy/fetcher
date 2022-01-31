use rss::Channel;

use crate::error::Error;
use crate::error::Result;
use crate::sink::Message;
use crate::source::Responce;

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
	pub async fn get(&mut self, last_read_id: Option<String>) -> Result<Vec<Responce>> {
		let content = self
			.http_client
			.get(&self.rss)
			.send()
			.await
			.map_err(|e| Error::SourceFetch {
				service: format!("RSS: {}", self.name),
				why: e.to_string(),
			})?
			.bytes()
			.await
			.map_err(|e| Error::SourceFetch {
				service: format!("RSS: {}", self.name),
				why: e.to_string(),
			})?;
		let mut feed = Channel::read_from(&content[..]).map_err(|e| Error::SourceParse {
			service: format!("RSS: {}", self.name),
			why: e.to_string(),
		})?;
		tracing::debug!("Got {amount} RSS articles", amount = feed.items.len());

		if let Some(id) = &last_read_id {
			if let Some(id_pos) = feed
				.items
				.iter()
				.position(|x| x.guid.as_ref().unwrap().value == id.as_str())
			{
				feed.items.drain(id_pos..);
			}
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

				Responce { id: Some(x.guid.as_ref().unwrap().value.clone()), msg: Message { text, media: None }}
			})
			.collect();

		Ok(messages)
	}
}
