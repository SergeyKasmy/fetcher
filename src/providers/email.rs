use mailparse::ParsedMail;

use crate::error::Result;
use crate::guid::Guid;
use crate::telegram::Message;

const IMAP_PORT: u16 = 993;

pub struct Email {
	imap: &'static str,
	email: &'static str,
	password: String,
}

impl Email {
	pub fn new(imap: &'static str, email: &'static str, password: String) -> Self {
		Self {
			imap,
			email,
			password,
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
			.map(|x| Self::parse_mail(mailparse::parse_mail(x.body().unwrap()).unwrap()))
			.collect::<Vec<_>>())
	}

	fn parse_mail(mail: ParsedMail) -> Message {
		let (subject, body) = {
			let subject = mail.headers.iter().find_map(|x| {
				if x.get_key_ref().to_ascii_lowercase() == "subject" {
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
