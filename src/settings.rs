mod config;
mod data;
mod last_read_id;

pub use self::config::config;
pub use self::data::{
	generate_google_oauth2, generate_google_password, generate_telegram, generate_twitter_auth,
	google_oauth2, google_password, telegram, twitter,
};
pub use self::last_read_id::{last_read_id, save_last_read_id};

const PREFIX: &str = "fetcher";
