# fetcher

fetcher is a flexible async framework designed to make it easy to create robust applications for building data pipelines to extract, transform, and deliver data from various sources to diverse destinations.
In easier words, it makes it easy to create an app that periodically checks a source, for example a website, for some data, makes it pretty, and sends it to the users.

fetcher is made to be easily extensible to support as many use-cases as possible while providing tools to support most of the common ones out of the box.

## Architecture

At the heart of fetcher is the [`Task`](`crate::task::Task`). It represents a specific instance of a data pipeline which consists of 2 main stages:

* [`Source`](`crate::sources::Source`): Fetches data from an external source (e.g. HTTP endpoint, email inbox).
* [`Action`](`crate::actions::Action`): Applies transformations (filters, modifications, parsing) to the fetched data.
The most notable action is [`Sink`](`crate::sinks::Sink`) that sends the transformed data somewhere (e.g. Discord channel, Telegram chat, another program's stdin)

An [`Entry`](`crate::entry::Entry`) is the unit of data flowing through the pipeline. It most notably contains:

* [`id`](`crate::entry::Entry::id`): A unique identifier for the entry, used for tracking read/unread status and replies.
* [`raw_contents`](`crate::entry::Entry::raw_contents`): The raw, untransformed data fetched from the source.
* [`msg`](`crate::entry::Entry::msg`): A [`Message`](`crate::sinks::message::Message`) that contains the formated and structured data, like title, body, link, that will end up sent to a sink.

A [`Job`](`crate::job::Job`) is a collections of one or more tasks that are executed together, potentially on a schedule.
Jobs can also be run either concurrently or in parallel (depending on the "send" feature) as a part of a [`JobGroup`](`crate::job::JobGroup`).

## Getting started

To use fetcher, you need to add it as a dependency to your `Cargo.toml` file:

```toml
[dependencies]
fetcher = { version = "0.15", features = ["full"] }
tokio = { version = "1", features = ["full"] }
```

For the smallest example on how to use fetcher, please see `examples/simple_website_to_stdout.rs`.
More complete examples can be found in the `examples/` directory. They demonstrate how toj

* Fetch data from various sources.
* Transform and filter data using regular expressions, HTML parsing, JSON parsing.
* Implement custom sources, actions, sinks
* Persist the read filter state in an external storage system

## Features

Each source, action, and sink (which is also an action but different enough to warrant being separate),
is gated behind a feature gate to help on the already pretty bad build times for apps using fetcher.

A feature is usually named using "(source|action|sink)-(name)" format.
Not only that, all sources, actions, and sinks (and misc features like `google-oauth2`) are also grouped into "all-(sources|actions|sinks|misc)" features
to enable every source, action, sink, or misc respectively.

Every feature can be enabled with the feature `full`.
This is the preffered way to use fetcher for the first time as it enables to use everything you might need before you actually know what you need.
Later on `full` can be replaced with the actual features you use to get some easy compile time gains.

For example, an app fetching RSS feeds and sending them to a telegram channel might use features `source-http`, `action-feed`, and `sink-telegram`.

## Note

fetcher was completely rewritten in v0.15.0.
It changed from an application with a config file to an application framework.

This was mostly done to make using fetcher correctly as easy and bug-free as possible.
Not to mention the huge config file was getting unwieldy and difficult to write and extend to your needs.
To make the config file more flexible would require integrating an actual programming language into it (like Lua).
I actually considered integrating Lua into the config file (a-la the Astral web framework) before I remembered that
we already have a properly integrated programming language, the one `fetcher` has always been written in in the first place.

I decided to double down on the fact that `fetcher` is written in Rust,
instead making `fetcher` a highly-extensible easy-to-use generic automation and data pipelining framework
which can be used to build apps, including apps similar to what `fetcher` has originally been.

Since then `fetcher-core` and `fetcher-config` crates are no longer used (or needed),
so if anybody needs these on crates.io, hit me up!

## Contributing

Contributions are very welcome! Please feel free to submit a pull request or open issues for any bugs, feature requests, or general feedback.

License: MPL-2.0
