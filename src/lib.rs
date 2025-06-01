/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! fetcher is a flexible async framework designed to make it easy to create robust applications for building data pipelines to extract, transform, and deliver data from various sources to diverse destinations.
//! In easier words, it makes it easy to create an app that periodically checks a source, for example a website, for some data, makes it pretty, and sends it to the users.
//!
//! fetcher is made to be easily extensible to support as many use-cases as possible while providing tools to support most of the common ones out of the box.
//!
//! # Architecture
//!
//! At the heart of fetcher is the [`Task`](`crate::task::Task`). It represents a specific instance of a data pipeline which consists of 3 main stages:
//!
//! * [`Source`](`crate::sources::Source`): Fetches data from an external source (e.g. HTTP endpoint, email inbox).
//! * [`Action`](`crate::actions::Action`): Applies transformations (filters, modifications, parsing) to the fetched data.
//! * [`Sink`](`crate::sinks::Sink`): Sends the transformed data to a destinations (e.g. Discord channel, Telegram bot, another program's stdin).
//!
//! An [`Entry`](`crate::entry::Entry`) is the unit of data flowing through the pipeline. It contains:
//!
//! * [`id`](`crate::entry::Entry::id`): A unique identifier for the entry, used for tracking read/unread status and replies.
//! * [`raw_contents`](`crate::entry::Entry::raw_contents`): The raw, untransformed data fetched from the source.
//! * [`msg`](`crate::entry::Entry::msg`): A [`Message`](`crate::sinks::message::Message`) that contains the formated and structured data, like title, body, link, that will end up sent to a sink.
//!
//! A [`Job`](`crate::job::Job`) is a collections of tasks that are executed together, potentially on a schedule.
//! Jobs can also be run either concurrently or in parallel as a part of a [`JobGroup`](`crate::job::JobGroup`).
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
//! * Send data to sinks like Telegram and Discord
//! * Implement custom sources, actions, sinks
//! * Persist the read filter state in an external storage system
//!
//! # Contributing
//!
//! Contributions are very welcome! Please feel free to submit a pull request or open issues for any bugs, feature requests, or general feedback.

#![cfg_attr(feature = "nightly", feature(never_type))]
#![cfg_attr(not(feature = "send"), expect(clippy::future_not_send))]
#![cfg_attr(test, allow(clippy::unwrap_used))]
// TODO: enable later
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

pub mod actions;
pub mod auth;
pub mod ctrl_c_signal;
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

// TODO: used to installa CryptoProvider. Not sure how this can be avoided
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
		job::{HandleError, OpaqueJob},
		maybe_send::MaybeSync,
		read_filter::ReadFilter,
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
				+ ReadFilter
				+ Sink
				+ Source
				+ Transform
				+ TransformField,
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
