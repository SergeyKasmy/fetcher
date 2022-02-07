# fetcher

fetcher makes it easier to see any information, like news or articles, in a place most comfortable to you.
It fetches any info you choose from a list of available sources, e.g. RSS, Twitter, email; filters it, and sends it to you.
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

The config is a toml file located at `$XDG_CONFIG_HOME/fetcher/config.toml` and has these mandatory fields

```
[<source_name>]
type = <source_type>
```

Other fields are type dependant and thus will be asked for when first run.

To set up login credentials, run fetcher with `--gen-secret--<name>` where name is either of these services:

* `google-oauth2`
* `twitter`
* `telegram`

After that you will be able to use these services automatically without additional authorization.
There's also a way to use an app password for Google/Gmail (saved with `--gen-secret-google-password`) but it's insecure and shouldn't be used for anything other than testing purposes
