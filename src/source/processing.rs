pub mod html;
pub mod rss;

pub use self::html::Html;
pub use self::rss::Rss;

#[derive(Debug)]
pub enum Process {
	Html(Html),
	Rss(Rss),
}
