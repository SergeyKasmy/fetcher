/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Reddit`] subbreddit API source

use super::Fetch;
use crate::{
	entry::{Entry, EntryId},
	sinks::message::{Media, Message},
};

use non_non_full::NonEmptyVec;
use roux::{
	Subreddit,
	util::{FeedOption, TimePeriod},
};
use std::fmt::Debug;

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

impl Fetch for Reddit {
	type Err = RedditError;

	/// Fetches all posts from a subreddit
	///
	/// # Errors
	/// This function may error if the network connection is down, or Reddit API returns a bad or garbage responce
	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		// TODO: inline this fn
		self.fetch_impl().await
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

				let link = post.url;

				// TODO: don't igonre the clippy lint. Use a case insensetive ASCII search
				#[expect(clippy::case_sensitive_file_extension_comparisons)]
				let is_picture = link.as_ref().is_some_and(|link| link.ends_with(".jpg"));

				#[expect(clippy::case_sensitive_file_extension_comparisons)]
				let is_video = link.as_ref().is_some_and(|link| {
					link.ends_with(".mp4") || link.ends_with(".gif") || link.ends_with(".gifv")
				});

				// post.url instead of link to avoid an unnecessary string -> url -> string conv
				let mut body = match (post.is_self, is_picture, &link) {
					(true, _, _) => post.selftext,
					(_, false, Some(link)) => link.clone(),
					_ => String::new(),
				};

				body.insert_str(0, &format!("Score: {}\n\n", post.score));

				let media = if is_picture {
					let url = link.expect(
						"should contain a valid picture url since we confirmed it with is_picture",
					);

					Some(NonEmptyVec::new_one(Media::Photo(url)))
				} else if is_video {
					let url = link.expect(
						"should contain a valid picture url since we confirmed it with is_video",
					);

					Some(NonEmptyVec::new_one(Media::Video(url)))
				} else {
					None
				};

				let link = format!("https://reddit.com/{}", post.permalink);

				Some(Entry {
					id: EntryId::new(post.id),
					raw_contents: None,
					msg: Message {
						title: Some(post.title),
						body: Some(body),
						link: Some(link),
						media,
					},
					..Default::default()
				})
			})
			.collect::<_>();

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
