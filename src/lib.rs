/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! fetcher is a flexible async framework designed to make it easy to create robust applications for building data pipelines to extract,
//! transform, and deliver data from various sources to diverse destinations.
//! In easier words, it makes it easy to create an app that periodically checks a source, for example a website, for some data, makes it pretty, and sends it to the users.
//!
//! fetcher is made to be easily extensible to support as many use-cases as possible while providing tools to support most of the common ones out of the box.
//!
//! # Architecture
//!
//! At the heart of fetcher is the [`Task`](`crate::task::Task`). It represents a specific instance of a data pipeline which consists of 2 main stages:
//!
//! * [`Source`](`crate::sources::Source`): Fetches data from an external source (e.g. HTTP endpoint, email inbox).
//! * [`Action`](`crate::actions::Action`): Applies transformations (filters, modifications, parsing) to the fetched data.
//! The most notable action is [`Sink`](`crate::sinks::Sink`) that sends the transformed data somewhere (e.g. Discord channel, Telegram chat, another program's stdin)
//!
//! An [`Entry`](`crate::entry::Entry`) is the unit of data flowing through the pipeline. It most notably contains:
//!
//! * [`id`](`crate::entry::Entry::id`): A unique identifier for the entry, used for tracking read/unread status and replies.
//! * [`raw_contents`](`crate::entry::Entry::raw_contents`): The raw, untransformed data fetched from the source.
//! * [`msg`](`crate::entry::Entry::msg`): A [`Message`](`crate::sinks::message::Message`) that contains the formated and structured data,
//! like title, body, link, that will end up sent to a sink.
//!
//! A [`Job`](`crate::job::Job`) is a collections of one or more tasks that are executed together, potentially on a schedule.
//! Jobs can also be run either concurrently or in parallel (depending on the "send" feature) as a part of a [`JobGroup`](`crate::job::JobGroup`).
//!
//! ## fetcher is extensible
//!
//! Everything in fetcher is defined and used via traits, including but not limited to:
//! [`Jobs`](`crate::job::OpaqueJob`), [`Tasks`](`crate::task::OpaqueTask`),
//! [`Sources`](`crate::sources::Source`), [`Actions`](`crate::actions::Action`),
//! [`JobGroups`](`crate::job::JobGroup`).
//!
//! This allows you to define and use anything you might be missing in fetcher by default without having to modify any fetcher code whatsoever.
//!
//! The easiest way to extend fetcher's parsing capabilities is to use [`transform_fn`][transform_fn]
//! that allows you to just pass in an async closure that modifies entries in whatever way you might want.
//!
//! * Want to deserialize JSON into a struct with `serde` to get better error reporting and more flexibility than using [`Json`](`crate::actions::transforms::Json`)?
//! Easy-peasy, just use [`transform_fn`][transform_fn] to wrap an async closure
//! in which you just call `let deserialized: Foo = serde_json::from_str(&entry.raw_contents)` and use it however you want.
//! * Want to do a bunch of text manipulations and avoid a thousand
//! [`Replace's`](`crate::actions::transforms::field::Replace`) & [`Extract's`](`crate::actions::transforms::field::Extract`)?
//! [`transform_fn`][transform_fn] got your back, too.
//! * Current selection of sinks is not enough? Define your own by implementing the [`Sink`](`crate::sinks::Sink`) trait on your type.
//! * Don't like default read-filtering strategies? Implement [`MarkAsRead`](`crate::read_filter::MarkAsRead`)
//! and [`Filter`](`crate::actions::filters::Filter`) on your type.
//! * Want to keep read state of entries in a database or just on the filesystem?
//! Implement [`ExternalSave`](`crate::external_save::ExternalSave`) yourself and do whatever you want.
//!
//! If anything is *not* extensible, this is a bug and it should be reported.
//!
//! # Getting started
//!
//! To use fetcher, you need to add it as a dependency to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! fetcher = { version = "0.15", features = ["full"] }
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! For the smallest example on how to use fetcher, please see `examples/simple_website_to_stdout.rs`.
//! More complete examples can be found in the `examples/` directory. They demonstrate how to:
//!
//! * Fetch data from various sources.
//! * Transform and filter data using regular expressions, HTML parsing, JSON parsing.
//! * Implement custom sources, actions, sinks
//! * Persist the read filter state in an external storage system
//!
//! # Features
//!
//! ## send
//!
//! Use the (enabled by default) `send` feature to enable tokio multithreading support.
//!
//! If `send` is disabled, then the `Send + Sync` bounds are relaxed from most types
//! but job groups no longer run jobs in parallel, using [`tokio::task::spawn_local`] instead of [`tokio::spawn`].
//! Please note that this requires you to wrap your calls to [`JobGroup::run`](`crate::job::job_group::JobGroup::run`) in a [`tokio::task::LocalSet`] to work.
//! Please see `tests/non_send.rs` for an example.
//!
//! ## nightly
//!
//! The `nightly` feature enables some traits implementation for some Rust nightly-only types, like `!`.
//!
//! ## full and all-sources, all-actions, all-sinks, all-misc
//!
//! Each source, action, and sink (which is also an action but different enough to warrant being separate),
//! is gated behind a feature gate to help on the already pretty bad build times for apps using fetcher.
//!
//! A feature is usually named using "(source|action|sink)-(name)" format.
//! Not only that, all sources, actions, and sinks (and misc features like `google-oauth2`) are also grouped into "all-(sources|actions|sinks|misc)" features
//! to enable every source, action, sink, or misc respectively.
//!
//! Every feature can be enabled with the feature `full`.
//! This is the preffered way to use fetcher for the first time as it enables to use everything you might need before you actually know what you need.
//! Later on `full` can be replaced with the actual features you use to get some easy compile time gains.
//!
//! For example, an app fetching RSS feeds and sending them to a telegram channel might use features `source-http`, `action-feed`, and `sink-telegram`.
//!
//! # Note
//!
//! fetcher was completely rewritten in v0.15.0.
//! It changed from an application with a config file to an application framework.
//!
//! This was mostly done to make using fetcher correctly as easy and bug-free as possible.
//! Not to mention the huge config file was getting unwieldy and difficult to write and extend to your needs.
//! To make the config file more flexible would require integrating an actual programming language into it (like Lua).
//! I actually considered integrating Lua into the config file (a-la the Astral web framework) before I remembered that
//! we already have a properly integrated programming language, the one `fetcher` has always been written in in the first place.
//!
//! I decided to double down on the fact that `fetcher` is written in Rust,
//! instead making `fetcher` a highly-extensible easy-to-use generic automation and data pipelining framework
//! which can be used to build apps, including apps similar to what `fetcher` has originally been.
//!
//! Since then `fetcher-core` and `fetcher-config` crates are no longer used (or needed),
//! so if anybody needs these on crates.io, hit me up!
//!
//! # Contributing
//!
//! Contributions are very welcome! Please feel free to submit a pull request or open issues for any bugs, feature requests, or general feedback.
//!
//! [transform_fn]: `crate::actions::transform_fn`

// TODO: show required features on docs.rs using something like this (copied from tokio):
//             #[cfg(any(all(doc, docsrs), windows))]
//             #[cfg_attr(docsrs, doc(cfg(windows)))]

#![cfg_attr(not(feature = "send"), expect(clippy::future_not_send))]
#![cfg_attr(feature = "nightly", feature(never_type))]
#![cfg_attr(test, allow(clippy::unwrap_used))]

pub mod actions;
pub mod auth;
pub mod cancellation_token;
pub mod entry;
pub mod error;
pub mod exec;
pub mod external_save;
pub mod job;
pub mod maybe_send;
pub mod read_filter;
pub mod sinks;
pub mod sources;
pub mod task;
pub mod utils;

#[cfg(feature = "scaffold")]
pub mod scaffold;

// common types
pub use crate::{job::Job, task::Task};

// exports to avoid different dependecy versions errors
pub use either;
pub use non_non_full;
pub use staticstr::StaticStr;
pub use url;

// TODO: used to install a CryptoProvider. Not sure how this can be avoided
// pub use tokio_rustls::rustls::crypto as rustls_crypto;

pub(crate) mod safe_slice;

#[cfg(test)]
mod tests {
	use std::{convert::Infallible, marker::PhantomData};

	use crate::{
		actions::{
			Action,
			filters::Filter,
			transforms::{Transform, field::TransformField},
		},
		external_save::ExternalSave,
		job::{HandleError, OpaqueJob, Trigger},
		maybe_send::MaybeSync,
		read_filter::MarkAsRead,
		sinks::Sink,
		sources::Source,
		task::OpaqueTask,
	};

	#[test]
	#[ignore = "nothing to execute, just a compile test"]
	fn common_types_implement_main_traits() {
		struct ImplementsCommonTraits<T, Tr = ()> {
			t: PhantomData<T>,
			tr: PhantomData<Tr>,
		}

		impl<T, Tr> ImplementsCommonTraits<T, Tr>
		where
			T: Action
				+ ExternalSave
				+ Filter
				+ HandleError<Tr>
				+ OpaqueJob
				+ OpaqueTask
				+ MarkAsRead
				+ Sink
				+ Source
				+ Transform
				+ TransformField
				+ Trigger,
			Tr: MaybeSync,
		{
			fn test() {}
		}

		ImplementsCommonTraits::<()>::test();
		ImplementsCommonTraits::<Infallible>::test();
		ImplementsCommonTraits::<Option<()>>::test();
		ImplementsCommonTraits::<&mut ()>::test();

		#[cfg(feature = "nightly")]
		ImplementsCommonTraits::<!>::test();
	}
}
