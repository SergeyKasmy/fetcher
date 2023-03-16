# fetcher

fetcher makes it easier to automate any information gathering, like news or blogs, into a place most comfortable to you.
It fetches anything you choose (from a list of available sources, e.g. RSS); filters it, and sends it whereever you want it to.
Think if it like IFTTT but locally hosted and if it only supported transfering text.

I wrote fetcher for myself after IFTTT had become paid, not to mention that by that time I was already not satisfied with it. I want to receive news and notifications in one place without having to search for it.
For example, imagine I wanted to read tweets by somebody but only if it contained a particular string. Before I had to receive _all_ notifications from that user on my phone via the Twitter app and read every one of them to find those relevant to me.
That honestly sucks. After looking for some locally hosted alternatives to IFTTT, I found none that were both lightweight and useful for me, so I decided to write my own.

fetcher is both a binary, and a library crate, so it can do programmatically everything it usually can.

Feel free to contribute if you want a particular feature added.

## Install

Download and install from [crates.io](https://crates.io/crates/fetcher) with 

```
cargo install fetcher
```

or build manually with

```
git clone https://github.com/SergeyKasmy/fetcher.git
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
  rss:
    read_filter_type: newer_than_read	# save only the last one sent/read
    source:
      http: '<your_rss_feed_url>'
    process:
      - feed
      - read_filter	# leave out only entries newer than the last one read
    sink:
      discord:
        user: <your_user_id>
```

This job is run every 30 minutes and has a single task named "rss". This task gets the contents of the web page <your_rss_feed_url>, parses it as an RSS/Atom feed, removes all feed items that have already been read (using the "newer_than_read" stradegy), and sends all items left via DMs to a Discord user <your_user_id>.

Currently available sources:

* email
* twitter
* http
* file

actions:

* read_filter: filters out read entries
* take: takes any num of entries from the beginning or the end
* http: follows url
* html: parses html
* json: parses json
* feed: parses rss and atom feeds
* use_raw_contents: use unparsed data from a source as the body of the message, e.g. raw html
* print: debug prints current entry contents  (mostly for testing)
* set: set a field in a message to a predefined value or null (e.g. used when you don't care or have nothing to extract from the source)
* caps: make all message text uppercase (mostly for testing)
* trim: remove leading and trailing whitespace
* shorten: shorten a field to be no longer than the max len
* regex: extract string from a different string; filter out strings that don't match; replace a match with a replacement string (with capture group support)

sinks:

* telegram
* stdout

Since a lot of these fields are dependent on the particular source, parser, and sink types and since fetcher is in heavy development at the moment, there isn't any template or example config files but fetcher will notify you if there are missing fields and what values they can have, so it's not that difficult to make one by trial and error even without reading the source code.

### Login credentials

To set up login credentials, run fetcher in `save` mode, following by a service name which is either of these:

* `google-oauth`
* `twitter`
* `telegram`

After finishing the prompt, you will be able to use any of these services automatically without additional authorization.
There's also a way to use an app password for Gmail/IMAP (saved with `email-password`) but it's insecure and shouldn't be used for anything other than testing purposes
