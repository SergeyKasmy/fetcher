# fetcher

fetcher makes it easier to see any information, like news or articles, in a place most comfortable to you.
It fetches any info you choose from a list of available sources, e.g. RSS or Twitter; filters it, and sends it to you.
Think if it like IFTTT but locally hosted and if it only supported transfering info.

I wrote fetcher for myself after IFTTT had become paid, not to mention that by that time I was already not satisfied with it. I like to receive news and notifications in one place without having to search for it.
For example, imagine I wanted to read tweets by somebody but only if it contained a particular string. Before I had to receive _all_ notifications from that user on my phone and read every one of them to find those relevant to me.
That honestly sucks. After looking for some locally hosted alternatives to IFTTT, I found none that were both lightweight and useful for me, so I decided to write my own.
Currently it's way more specific for my personal usecase but I'm trying to make it more modular and general but it's still WIP.

Feel free to contribute if you want a particular feature/source/sink added.
I like to have my git history clean and easy to navigate through, so try to keep it this way, please :)

## Install

Download and install from [crates.io](https://crates.io) with 

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

The main unit of execution in fetcher is a task. A task consists of a source where to fetch some kind of data from, (a) parser(s) which process the data, and a sink where to send that data to. To create a task, create a `foo.yaml` file in `$XDG_CONFIG_HOME/fetcher/tasks` or `/etc/xdg/fetcher/tasks` where `foo` is the name you want that task to have. A proper task config file looks something like this:

```yaml
# optional
disabled: true

refresh: 30	 # in minutes
read_filter_type: newer_than_read	# save only the last one sent/read
source:
  http: '<your_rss_feed_url>'
transform:
  - rss
  - read_filter	# leave out only entries newer than the last one read
sink:
  telegram:
    chat_id: <your_telegram_chat_id>
```

Currently available source types:

* email
* twitter
* http
* file

transform types:

* http: follows url
* html: parses html
* rss: parses rss
* json: parses json
* read_filter: filters out read entries
* take: takes any num of entries from the beginning or the end
* use_raw_contents: use unparsed data from a source as the body of the message, e.g. raw html
* print: debug prints current entry contents  (mostly for testing)
* caps: make all message text uppercase (mostly for testing)

sink types:

* telegram
* stdout
* null

Since a lot of these fields are dependent on the particular source, parser, and sink types and since fetcher is in heavy development at the moment, there isn't any template or example config files but fetcher will notify you if there are missing fields and what values they can have, so it's not that difficult to make one by trial and error even without reading the source code.

### Login credentials

To set up login credentials, run fetcher with `--save-secret-<name>` where name is either of these services:

* `google-oauth2`
* `twitter`
* `telegram`

After finishing the prompt, you will be able to use any of these services automatically without additional authorization.
There's also a way to use an app password for Gmail/IMAP (saved with `--save-secret-email-password`) but it's insecure and shouldn't be used for anything other than testing purposes
