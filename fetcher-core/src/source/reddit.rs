/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Reddit`] subbreddit API source

use super::Fetch;
use crate::{
	entry::Entry,
	error::InvalidUrlError,
	sink::message::{Media, Message},
	source::error::SourceError,
	utils::OptionExt,
};

use async_trait::async_trait;
use roux::{
	Subreddit,
	util::{FeedOption, TimePeriod},
};
use std::fmt::Debug;
use url::Url;

/// Source that fetches posts from a subreddit using the Reddit API
pub struct Reddit {
	/// Sorting algorithm
	pub sort: Sort,
	/// If score of a post is below this threshold, it gets skipped
	pub score_threshold: Option<u32>,
	subreddit: Subreddit,
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum RedditError {
	#[error(transparent)]
	Reddit(#[from] roux::util::RouxError),

	#[error(
		"Reddit API returned an invalid URL to a post/post's contents, which really shouldn't happen..."
	)]
	InvalidUrl(#[from] InvalidUrlError),
}

/// Sorting algorithm
#[derive(Debug)]
pub enum Sort {
	/// Latest/New
	Latest,
	/// Rising
	Rising,
	/// Hot
	Hot,
	/// Top of the day
	TopDay,
	/// Top of the week
	TopWeek,
	/// Top of the month
	TopMonth,
	/// Top of the year
	TopYear,
	/// Top of all time
	TopAllTime,
}

impl Reddit {
	/// Creates a new [`Reddit`] source.
	#[must_use]
	pub fn new(subreddit: &str, sort: Sort, score_threshold: Option<u32>) -> Self {
		Self {
			sort,
			score_threshold,
			subreddit: Subreddit::new(subreddit),
		}
	}
}

#[async_trait]
impl Fetch for Reddit {
	/// Fetches all posts from a subreddit
	///
	/// # Errors
	/// This function may error if the network connection is down, or Reddit API returns a bad or garbage responce
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		self.fetch_impl().await.map_err(Into::into)
	}
}

impl Reddit {
	async fn fetch_impl(&self) -> Result<Vec<Entry>, RedditError> {
		let s = &self.subreddit;

		macro_rules! top_in {
			($tp:tt) => {
				s.top(
					100,
					Some(FeedOption::new().limit(100).period(TimePeriod::$tp)),
				)
				.await
			};
		}

		let posts = match self.sort {
			Sort::Latest => s.latest(100, None).await,
			Sort::Rising => s.rising(100, None).await,
			Sort::Hot => s.hot(100, None).await,
			Sort::TopDay => top_in!(Today),
			Sort::TopWeek => top_in!(ThisWeek),
			Sort::TopMonth => top_in!(ThisMonth),
			Sort::TopYear => top_in!(ThisYear),
			Sort::TopAllTime => top_in!(AllTime),
		}?;

		let entries = posts
			.data
			.children
			.into_iter()
			.filter_map(|post| {
				let post = post.data;

				if let Some(score_threshold) = self.score_threshold {
					if post.score < score_threshold.into() {
						return None;
					}
				}

				let link = match post
					.url
					.as_deref()
					.try_map(|u| Url::try_from(u).map_err(|e| InvalidUrlError(e, u.to_owned())))
				{
					Ok(v) => v,
					Err(e) => return Some(Err(e)),
				};

				// TODO: don't igonre the clippy lint. Use a case insensetive ASCII search
				#[expect(clippy::case_sensitive_file_extension_comparisons)]
				let is_picture = link.as_ref().is_some_and(|u| u.path().ends_with(".jpg"));

				#[expect(clippy::case_sensitive_file_extension_comparisons)]
				let is_video = link.as_ref().is_some_and(|u| {
					let p = u.path();
					p.ends_with(".mp4") || p.ends_with(".gif") || p.ends_with(".gifv")
				});

				// post.url instead of link to avoid an unnecessary string -> url -> string conv
				let mut body = match (post.is_self, is_picture, post.url) {
					(true, _, _) => post.selftext,
					(_, false, Some(link)) => link,
					_ => String::new(),
				};

				body.insert_str(0, &format!("Score: {}\n\n", post.score));

				let media = if is_picture {
					let url = link.expect(
						"should contain a valid picture url since we confirmed it with is_picture",
					);

					Some(vec![Media::Photo(url)])
				} else if is_video {
					let url = link.expect(
						"should contain a valid picture url since we confirmed it with is_video",
					);

					Some(vec![Media::Video(url)])
				} else {
					None
				};

				let link = match format!("https://reddit.com/{}", post.permalink)
					.as_str()
					.try_into()
					.map_err(|e| InvalidUrlError(e, post.permalink))
				{
					Ok(v) => v,
					Err(e) => return Some(Err(e)),
				};

				Some(Ok(Entry {
					id: Some(post.id.into()),
					raw_contents: None,
					msg: Message {
						title: Some(post.title),
						body: Some(body),
						link: Some(link),
						media,
					},
					..Default::default()
				}))
			})
			.collect::<Result<_, _>>()?;

		Ok(entries)
	}
}

impl Debug for Reddit {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Reddit")
			.field("subreddit", &self.subreddit.name)
			.field("sort", &self.sort)
			.field("score_threshold", &self.score_threshold)
			.finish()
	}
}
