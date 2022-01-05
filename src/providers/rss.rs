use crate::error::Error;
use crate::error::Result;
use crate::guid::Guid;
use crate::telegram::Message;

use rss::Channel;

#[derive(Debug)]
pub struct Rss {
	name: &'static str,
	rss: &'static str,
	http_client: reqwest::Client,
}

impl Rss {
	pub fn new(name: &'static str, rss: &'static str) -> Self {
		Self {
			name,
			rss,
			http_client: reqwest::Client::new(),
		}
	}

	pub async fn get(&mut self) -> Result<Vec<Message>> {
		let mut last_read_guid = Guid::new(self.name)?;
		let content = self
			.http_client
			.get(self.rss)
			.send()
			.await
			.map_err(|e| Error::Get {
				service: format!("RSS: {}", self.name),
				why: e.to_string(),
			})?
			.bytes()
			.await
			.map_err(|e| Error::Get {
				service: format!("RSS: {}", self.name),
				why: e.to_string(),
			})?;
		let mut feed = Channel::read_from(&content[..]).map_err(|e| Error::Parse {
			service: format!("RSS: {}", self.name),
			why: e.to_string(),
		})?;

		if let Some(last_read_guid_pos) = feed
			.items
			.iter()
			.position(|x| x.guid.as_ref().unwrap().value == last_read_guid.guid)
		{
			feed.items.drain(last_read_guid_pos..);
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

				last_read_guid.guid = x.guid.as_ref().unwrap().value.clone(); // NOTE: crash if the feed item doesn't have a guid. That should never happen though
				Message { text, media: None }
			})
			.collect();

		last_read_guid.save()?;

		Ok(messages)
	}
}
