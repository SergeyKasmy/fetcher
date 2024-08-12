# fetcher

fetcher makes it easier to automate any information gathering, like news or articles from blogs, into a place most comfortable to you.
It fetches anything you choose (from a list of available sources); processes it (filters and parses), and sends it wherever you want it to.
Think of it like IFTTT but locally hosted and if it only supported transferring text.

I wrote fetcher for myself after IFTTT had become paid, not to mention that by that time I was already not satisfied with it. I want to receive news and notifications in one place without having to search for it.
For example, imagine I wanted to read tweets by somebody but only if it contained a particular string. Before I had to receive _all_ notifications from that user on my phone via the Twitter app and read every one of them to find those relevant to me.
That honestly sucks. After looking for some locally hosted alternatives to IFTTT, I found none that were both lightweight and useful for me, so I decided to write my own.

fetcher is both a binary, and a [library](https://docs.rs/fetcher-core/latest/fetcher_core) crate, so it can do programmatically everything it usually can.

Feel free to contribute if you want a particular feature added.

## Install

Download and install from [crates.io](https://crates.io/crates/fetcher) with 

```
cargo install fetcher
```

or build manually with

```
git clone -b main --single-branch https://github.com/SergeyKasmy/fetcher.git
cd fetcher
cargo build --release
```

The final binary will be located in `target/release/fetcher` which you can then copy to `~/.local/bin` or any other dir included in your `$PATH`.

## Setup

The main unit of execution in fetcher is a job. A job consists of one or more tasks that are rerun every set interval or once a day at a particular time. A task contains a source where to fetch the data from, (a) action(s) which process the data (modify, filter, remove already read), and a sink where the data is later sent to. To create a job, create a `foo.yml` file in `$XDG_CONFIG_HOME/fetcher/jobs` or `/etc/xdg/fetcher/jobs` where `foo` is the name you want that job to have. A proper job config file looks something like this:

```yaml
refresh: 
  every: 30m
tasks:
  news:
    read_filter_type: newer_than_read
    source:
      twitter: '<your twitter handle>'
    process:
      - read_filter # leave out only entries newer than the last one read
      - contains:
          body: '[Hh]ello'
      - set:
         title: New tweet from somebody
      - shorten:
          body: 50
    sink:
      discord:
        user: <your user id>
```

This job is run every 30 minutes and has a single task named "news". This task:
* gets the Twitter timeline of @<your twitter handle>
* removes all tweets that have already been read (using the `newer_than_read` stradegy)
* retains only tweets that contains "Hello" or "hello" in them
* sets the title to "New tweet from somebody"
* shortens the body to 50 characters if it is longer
* and sends all tweets left via DMs to a Discord user `<your_user_id>`.

## Running

Run fetcher with `fetcher run`. This will run all jobs found in all config locations. fetcher searches for all `.yml` jobs first in `$XDG_CONFIG_HOME/fetcher/jobs`, and then in `/etc/xdg/fetcher/jobs`.

You can specify a job manually in the commandline using JSON when run with `fetcher run-manual`

See `fetcher --help` for more details

### Login credentials

To set up login credentials, run fetcher in save mode (`fetcher save`), following by a service name which is either of these:

* `google-oauth2`
* `twitter`
* `telegram`
* `email-password`

After finishing the prompt, you will be able to use any of these services automatically without additional authorization.

## Job config format

To see all available config options, see [config-format.md](/config-format.md)
