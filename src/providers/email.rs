use mailparse::ParsedMail;

use crate::error::Result;
use crate::telegram::Message;

const IMAP_PORT: u16 = 993;

pub enum EmailFilter {
	Subject(&'static str),
	Sender(&'static str),
}

pub struct Email {
	imap: &'static str,
	email: String,
	password: String,
	filters: Option<&'static [EmailFilter]>,
	archive: bool,
	remove_after: Option<&'static str>, // NOTE: remove everything after this text, including itself, from the message
}

impl Email {
	pub fn new(
		imap: &'static str,
		email: String,
		password: String,
		filters: Option<&'static [EmailFilter]>,
		archive: bool,
		remove_after: Option<&'static str>,
	) -> Self {
		Self {
			imap,
			email,
			password,
			filters,
			archive,
			remove_after,
		}
	}

	pub async fn get(&mut self) -> Result<Vec<Message>> {
		let tls = native_tls::TlsConnector::builder().build().unwrap();
		let client = imap::connect((self.imap, IMAP_PORT), self.imap, &tls).unwrap();

		let mut session = client.login(&self.email, &self.password).unwrap();
		session.select("INBOX").unwrap();

		let search_string = {
			let mut tmp = "UNSEEN ".to_string();

			if let Some(filters) = self.filters {
				for f in filters {
					match f {
						EmailFilter::Subject(s) => {
							// TODO: mb use format!()?
							tmp.push_str(r#"SUBJECT ""#);
							tmp.push_str(s);
							tmp.push_str(r#"" "#);
						}
						EmailFilter::Sender(s) => {
							tmp.push_str(r#"FROM ""#);
							tmp.push_str(s);
							tmp.push_str(r#"" "#);
						}
					}
				}
			}

			tmp.trim_end().to_string()
		};
		let mail_ids = session
			.uid_search(search_string)
			.unwrap()
			.into_iter()
			.map(|x| x.to_string())
			.collect::<Vec<_>>()
			.join(",");

		if mail_ids.is_empty() {
			return Ok(Vec::new());
		}

		let mails = session.uid_fetch(&mail_ids, "BODY[]").unwrap();

		// TODO: don't archive if there were any errors while sending
		if self.archive {
			session
				.uid_store(&mail_ids, "+FLAGS.SILENT (\\Deleted)")
				.unwrap();
			session.uid_expunge(&mail_ids).unwrap();
		}

		session.logout().unwrap();

		Ok(mails
			.into_iter()
			.filter(|x| x.body().is_some()) // TODO: properly handle error cases and don't just filter them out
			.map(|x| {
				Self::parse(
					mailparse::parse_mail(x.body().unwrap()).unwrap(),
					self.remove_after,
				)
			})
			.collect::<Vec<_>>())
	}

	fn parse(mail: ParsedMail, remove_after: Option<&'static str>) -> Message {
		let (subject, body) = {
			let subject = mail.headers.iter().find_map(|x| {
				if x.get_key_ref() == "Subject" {
					Some(x.get_value())
				} else {
					None
				}
			});

			let mut body = if mail.subparts.is_empty() {
				&mail
			} else {
				mail.subparts
					.iter()
					.find(|x| x.ctype.mimetype == "text/plain")
					.unwrap_or(&mail.subparts[0])
			}
			.get_body()
			.unwrap();

			if let Some(remove_after) = remove_after {
				body.drain(body.find(remove_after).unwrap_or(body.len())..);
			}

			// TODO: replace upticks ` with teloxide::utils::html::escape_code

			// NOTE: emails often contain all kinds of html or other text which Telegram's HTML parser doesn't approve of
			// I dislike the need to add an extra dependency just for this simple task but you gotta do what you gotta do.
			// Hopefully I'll find a better way to escape everything though since I don't fear a possibility that it'll be
			// somehow harmful 'cause it doesn't consern me, only Telegram :P
			(subject, ammonia::clean(&body))
		};

		let text = match subject {
			Some(subject) => format!("{}\n\n{}", subject, body),
			None => body,
		};

		Message { text, media: None }
	}
}
