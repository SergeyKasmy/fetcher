use mailparse::ParsedMail;

use crate::error::Result;
use crate::guid::Guid;
use crate::telegram::Message;

const IMAP_PORT: u16 = 993;

pub enum EmailFilter {
	Subject(&'static str),
	Sender(&'static str),
}

pub struct Email {
	imap: &'static str,
	email: &'static str,
	password: String,
	filters: Option<&'static [EmailFilter]>,
}

impl Email {
	pub fn new(
		imap: &'static str,
		email: &'static str,
		password: String,
		filters: Option<&'static [EmailFilter]>,
	) -> Self {
		Self {
			imap,
			email,
			password,
			filters,
		}
	}

	pub async fn get(&mut self) -> Result<Vec<Message>> {
		let tls = native_tls::TlsConnector::builder().build().unwrap();
		let client = imap::connect((self.imap, IMAP_PORT), self.imap, &tls).unwrap();

		let mut session = client.login(self.email, &self.password).unwrap();
		session.examine("INBOX").unwrap();

		let unread_ids = session.uid_search("UNSEEN").unwrap();
		let mails = session
			.uid_fetch(
				unread_ids
					.into_iter()
					.map(|x| x.to_string())
					.collect::<Vec<_>>()
					.join(","),
				"BODY[]",
			)
			.unwrap();

		session.logout().unwrap();

		Ok(mails
			.into_iter()
			.map(|x| mailparse::parse_mail(x.body().unwrap()).unwrap())
			.filter(|x| Self::filter(x, self.filters.clone()))
			.map(|x| Self::parse(x))
			.collect::<Vec<_>>())
	}

	fn filter(mail: &ParsedMail, filters: Option<&'static [EmailFilter]>) -> bool {
		if let Some(filters) = filters {
			for filter in filters {
				if match filter {
					EmailFilter::Subject(s) => mail
						.headers
						.iter()
						.find(|x| x.get_key_ref() == "Subject" && x.get_value().contains(s)),
					EmailFilter::Sender(s) => mail
						.headers
						.iter()
						.find(|x| x.get_key_ref() == "From" && x.get_value().contains(s)),
				}
				.is_some()
				{
					return true;
					// NOTE: I can't just return match... .is_some() because I want to return early only in case it's true but not when it's false
					// Maybe there's a different way that I'm missing?
				}
			}
		}

		false
	}

	fn parse(mail: ParsedMail) -> Message {
		let (subject, body) = {
			let subject = mail.headers.iter().find_map(|x| {
				if x.get_key_ref() == "Subject" {
					Some(x.get_value())
				} else {
					None
				}
			});

			let body = if mail.subparts.is_empty() {
				&mail
			} else {
				mail.subparts
					.iter()
					.find(|x| x.ctype.mimetype == "text/plain")
					.unwrap_or(&mail.subparts[0])
			}
			.get_body()
			.unwrap();

			(subject, body)
		};

		Message {
			text: match subject {
				Some(subject) => format!("{}\n\n{}", subject, body),
				None => body,
			},
			media: None,
		}
	}
}
