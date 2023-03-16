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

This job is run every 30 minutes and has a single task named "rss". This task gets the contents of the web page `<your_rss_feed_url>`, parses it as an RSS/Atom feed, removes all feed items that have already been read (using the `newer_than_read` stradegy), and sends all items left via DMs to a Discord user `<your_user_id>`.

## All available config options

Note: options with a X after them are exclusive with each other and are not intended to be run simultaniously but are rather just listed with all their available options in the way they are supposed to be used.
Note: options with a O are optional

```yaml
disabled: true  # O
read_filter_type: newer_than_read # XO. either: 
                                  # * keep only the last read entry and filter out all "older" than it
                                  # * notify when the entry is updated
read_filter_type: not_present_in_read_list # XO. keep a list of all items read and filter out all that are present in it
tasks:
  foo:
    tag: <string> # mark the message with a tag. That is usually a hashtag on top of the message or some kind of subscript in it. If a job has multiple tasks, it is automatically set to the task's name
    source:
      string: <string> # X. set the body of an entry to set string
      http: # X
        - <url> # get the contents of a web page
        - <url> # or several ones. Note: they have compatible contents and IDs to be able to work with the processing and read filtering logic. If they do not, just create a different task
        - post: # send a POST request (instead of a GET request)
            url: <url>
            body: <string> # with its body set to <string>
      twitter: # X
        - <twitter_handle> # get the feed of a tweeter account
        - <twitter_handle> # or several
      file: # X
        - <path> # get the contents of a file
        - <path> # or several
      reddit: # X
        <subreddit_name>:
          sort: <new|rising|hot> # X
          sort: # X
            top: <today|thisweek|thismonth|thisyear|alltime> 
          score_threshhold: <int> # O. Ignore posts with score lower than the threshhold
        <subreddit_name>:	# can be specified multiple times
          ...
      exec: # X
        - <cmd> # exec this command and use its output
        - <cmd> # or several commands
      email: # X
        auth: <google_oauth2|password> # how to authenticate with the IMAP server. `password` is insecure. `google_oauth2` can only be used with Gmail
        imap: <url> # URL of the IMAP server. Used only with `auth: password`. With `auth: google_oauth2` `imap.gmail.com` is used automatically
        email: <address> # email address to authenticate with
        filters: # O
          sender: <email_address>  # O. Ignore all email not sent from this address
          subjects: # O
            - <string> # ignore all emails not containing this string
            - <string> # or several
          exclude_subjects: # O
            - <string> # ignore all emails containing this string
            - <string> # or several
        view_mode: <read_only|mark_as_read|delete>  # how to view the inbox.
                                                    # * read_only: doesn't modify the inbox in any way (but will get the same emails over and over again with no way to check which are read. Should be used with a `read_filter`)
                                                    # * mark_as_read: mark read emails as read
                                                    # * delete: move the emails to the trash bin. Exact behavior depends on the email provider in question. Gmail archives the emails by default instead
    process:  # all actions are optional, so don't need to be marked with O
      - read_filter # filter out already read entries using `read_filter_type` stradegy
      - take: # take `num` entries from either beginning or the end and ignore the rest
          from: <beginning|end>
          num: <int>
      - contains: # filter out all entries that don't match
          re: <regex> # regular expression to match the contents of the `field` against
          field: <field> # the field to match against. Refer later for which fields are valid
      - feed # parse the entries as an RSS/Atom feeds
      - html: # parse the entries as HTML. All queries use the same format, except for `item_query`
          item_query: # O. Item is a unit of information. For example, articles in a blog or goods in an online store search are items. If the entire page is the "item", then this should be ignored
            # either of tag|class|attr can be used any number of times. They specify a narrowing down traversal of the HTML that specifies an item. Refer to [docs.rs of ElementDataQuery](https://docs.rs/fetcher-core/latest/fetcher_core/action/transform/entry/html/query/struct.ElementDataQuery.html) for more details
            - tag: <string>
            - class: <string>
            - attr:
                <attr>: <value>	# look for match of `<html_attribute>` inside `<attr>`, i.e. `href: THIS` will match for the contents of <a href="THIS">foo</a>
          title_query: # O. A query to get the title of the entry from. Seaches inside the item found in "item query" if it set, the entire page otherwise
            optional: <bool> # defines what happens when this query doesn't match anything. if 'true', the title should be left empty, if 'false', the entire task will fail. `false` by default
            query:
              ... # the same as `itemq` above
            data_location: text # X. Where to exact data from. `text` extracts the text of an HTML tag, i.e. `<a href="https://example.com">THIS</a>`
            data_location: 
              attr: <string> # X. While `attr` extracts the contents of the attribute, i.e. `attr: href` extracts `<a href="THIS">and not this!</a>`
            regex: # O. match the resulting data got from this query against a regex
              re: <regex> # the regex to match against
              replace_with: <string> # replace the matched regex with this string. Supports referencing capture groups from `re`.
              # Example
              #   regex:
              #     re: '/.*/.*'
              #     replace_with: `Hello, $1!`
              # This regex extracts the data from `/HERE/not here/or here` and replaces the entire title with "Hello, HERE!"
          text_query: # O. Query for the main content of the message. 
            - ... # Same as `title_query` but is an array. This makes it possible to extract text from several different places and concatenate it into a single message body.
            - ...
          id_query: # O. Query for the ID of the item.
            ... # same as `title_query`
          link_query: # O. Query for the URL of the item. The entry 
            ... # same as `title_query`
          img_query: # O. "Query for the attached pictures of the item.
            ... # same as `title_query`
      - http # fetch a page from the link field of the message. Allows recursive web parsing.
      - json: # very similar to `html`
          item_query: # O. "Item query". Item is a unit of information. For example, articles in a blog or goods in an online store search are items. If the entire JSON is the "item", then this should be ignored
            query: # query that should be matched one by one to traverse the JSON and find the item
              - <string> # matches a JSON key
              - <int> # matches an item of an array or a map
          title_query: # O. A query to get the title of the entry from. Seaches inside the item found in "item query" if it set, the entire JSON otherwise
            optional: <bool> # defines what happens when this query doesn't match anything. if 'true', the title should be left empty, if 'false', the entire task will fail. `false` by default
            query:
              ... # the same as `itemq.query` above
            regex: # O. match the resulting data got from this query against a regex
              re: <regex> # the regex to match against
              replace_with: <string> # replace the matched regex with this string. Supports referencing capture groups from `re`.
              # Example
              #   regex:
              #     re: '/.*/.*'
              #     replace_with: `Hello, $1!`
              # This regex extracts the data from `/HERE/not here/or here` and replaces the entire title with "Hello, HERE!"
          text_query: # O. "Text query". Query for the main content of the message. 
            - ... # Same as `title_query` but is an array. This makes it possible to extract text from several different places and concatenate it into a single message body.
            - ...
          id_query: # O. "ID query". Query for the ID of the item.
            ... # same as `title_query`
          link_query: # O. "Link query". Query for the URL of the item. The entry 
            ... # same as `title_query`
          img_query: # O. "Image query". Query for the attached pictures of the item.
            ... # same as `title_query`
      - use:  # copy the data of a field to a different field of a message
          <field>:  # the field to copy the data from
            as: <field> # the field to copy the data to
          <field>:  # can be specified multiple times
            as: <field>
        # Example: 
        #   use:
        #     title:
        #       as: body
        # This will use the title of the message as the body of the message, i.e. they will be the same
      - set: # set a field to a specified string
          <field>: <string>  # set <field> to <string>
          <field>: <string>  # can be specified multiple times
      - shorten: # limit the length of a field to a specified maximum amount of charachers
          <field>: <int>  # limit <field> to <int> max charachers
          <field>: <int>  # can be specified multiple times
      - trim: <field> # remove leftover whitespace to the left and to the right of the contents of the <field>
      - replace: # replace the contents of a field
          re: <regex> # replace the first regex match
          field: <field> # in the field
          with: <string> # with this string
      - extract: # extract text using a regex
          from_field: <field> # extract text from this field and replace the contents of the field with it
          re: <regex> # the regex that specifies a capture group named "e" that will becone the new contents of the field
          passthrough_if_not_found: <bool> # what to do if the regex didn't match. If `true`, the value of the field `from_field` should remain the same, if `false`, the task will be aborted
      # debug related actions:
      - caps # make the message title uppercase
      - debug_print # debug print the entire contents of the entry
sink:
  discord: # X. Send as a discord message
    user: <user_id> # X. The user to DM to. This is not a handle (i.e. not User#1234) but rather the ID (see below). 
    channel: <channel_id> # X. The channel to send messages to
    # The ID of a user or a channel can be gotten after enabling developer settings in Discord (under Settings -> Advanced) and rightclicking on a user/channel and selecting "Copy ID"
  telegram: # X
    chat_id: <chat_id>  # Either the private chat (group/channel) ID that can be gotten using bots or the public handle of a chat. DM aren't supported yet.
    link_location: <prefer_title|bottom>  # O. Where to put the link. Either as try to put it in the title if it's present, or a separate "Link" button under the message
  exec: <cmd> # X. Start a process and write the body of the message to its stdin
  stdout # X. Just print to stdout. Isn't really useful but it is the default when run with --dry-run
```

Here are all currently available sources:

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
