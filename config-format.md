## All available config options

Note: options with an X after them are exclusive with each other and are not intended to be run simultaniously but are rather just listed with all their available options in the way they are supposed to be used.

Note: options with an O are optional

```yaml
disabled: true # O
read_filter_type: newer_than_read # XO. either: 
                                  # * keep only the last read entry and filter out all "older" than it
                                  # * notify when the entry is updated
read_filter_type: not_present_in_read_list # XO. keep a list of all items read and filter out all that are present in it
template: <name> # copy-paste the contents of $XDG_CONFIG_PATH/fetcher/templates/<name>.yml. Field re-definition overrides the old value. 
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
      - import: <name> # import a list of actions from $XDG_CONFIG_PATH/fetcher/actions/<name>.yml
      - sink:
          discord: # X. Send as a discord message
            user: <user_id> # X. The user to DM to. This is not a handle (i.e. not User#1234) but rather the ID (see below). 
            channel: <channel_id> # X. The channel to send messages to
            # The ID of a user or a channel can be gotten after enabling developer settings in Discord (under Settings -> Advanced) and rightclicking on a user/channel and selecting "Copy ID"
          telegram: # X
            chat_id: <chat_id>  # Either the private chat (group/channel) ID that can be gotten using bots or the public handle of a chat. DM aren't supported yet.
            link_location: <prefer_title|bottom>  # O. Where to put the link. Either as try to put it in the title if it's present, or a separate "Link" button under the message
          exec: <cmd> # X. Start a process and write the body of the message to its stdin
          stdout # X. Just print to stdout. Isn't really useful but it is the default when run with --dry-run
      - read_filter # filter out already read entries using `read_filter_type` stradegy
      - take: # take `num` entries from either the newest or the oldest and ignore the rest
          <from_newest|from_oldest>: <int>
      - contains: # filter out all entries that don't match
          <field>: <regex> # regular expression to match the contents of the <field> against
          <field>: <regex> # can be specified several times
      - feed # parse the entries as an RSS/Atom feeds
      - html: # parse the entries as HTML. All queries use the same format, except for `item_query`
          item: # O. Item is a unit of information. For example, articles in a blog or goods in an online store search are items. If the entire page is the "item", then this should be ignored
            query:
              # either of tag|class|attr can be used any number of times. They specify a narrowing down traversal of the HTML that specifies an item. Refer to [docs.rs of ElementDataQuery](https://docs.rs/fetcher-core/latest/fetcher_core/action/transform/entry/html/query/struct.ElementDataQuery.html) for more details
              - tag: <string>
              - class: <string>
              - attr:
                  <attr>: <value> # look for match of `<html_attribute>` inside `<attr>`, i.e. `href: THIS` will match for the contents of <a href="THIS">foo</a>
                ignore: # any of these can also include an `ignore` field that, in case several HTML tags matched the query, will ignore ones that match ~this~ ignore query
                  - tag: <string>
                  - class: <string>
                  - attr:
                      <attr>: <value>
          title: # O. A query to get the title of the entry from. Seaches inside the item found in "item query" if it set, the entire page otherwise
            optional: <bool> # defines what happens when this query doesn't match anything. if 'true', the title should be left empty, if 'false', the entire task will fail. `false` by default
            query:
              ... # the same as `item` above
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
          text: # O. Query for the main content of the message. 
            - ... # Same as `title` but is an array. This makes it possible to extract text from several different places and concatenate it into a single message body.
            - ...
          id: # O. Query for the ID of the item.
            ... # same as `title`
          link: # O. Query for the URL of the item. The entry 
            ... # same as `title`
          img: # O. "Query for the attached pictures of the item.
            ... # same as `title`
      - http # fetch a page from the link field of the message. Allows recursive web parsing.
      - json: # very similar to `html`
          item: # O. "Item query". Item is a unit of information. For example, articles in a blog or goods in an online store search are items. If the entire JSON is the "item", then this should be ignored
            query: # query that should be matched one by one to traverse the JSON and find the item
              - <string> # matches a JSON key
              - <int> # matches an item of an array or a map
          title: # O. A query to get the title of the entry from. Seaches inside the item found in "item query" if it set, the entire JSON otherwise
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
          text: # O. "Text query". Query for the main content of the message. 
            - ... # Same as `title` but is an array. This makes it possible to extract text from several different places and concatenate it into a single message body.
            - ...
          id: # O. "ID query". Query for the ID of the item.
            ... # same as `title`
          link: # O. "Link query". Query for the URL of the item. The entry 
            ... # same as `title`
          img: # O. "Image query". Query for the attached pictures of the item.
            ... # same as `title`
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
          <field>: <string> # set <field> to <string>
          <field>: <string> # can be specified multiple times
          <field>: 
            - <string> # or even as an array, in which case it will choose a random one each time
            - <string>
      - shorten: # limit the length of a field to a specified maximum amount of charachers
          <field>: <int> # limit <field> to <int> max charachers
          <field>: <int> # can be specified multiple times
      - trim: <field> # remove leftover whitespace to the left and to the right of every line in the <field>
      - replace: # replace the contents of a field
          re: <regex> # replace the first regex match
          field: <field> # in the field
          with: <string> # with this string
      - extract: # extract text using a regex
          from_field: <field> # extract text from this field and replace the contents of the field with it
          re: <regex> # the regex that specifies capture groups that will be concatenated and become the new contents of the field
          passthrough_if_not_found: <bool> # what to do if the regex didn't match. If `true`, the value of the field `from_field` should remain the same, if `false`, the task will be aborted
      - remove_html: # remove any HTML tags in <field> and trim any remaining whitespace
          in: <field> # X. either in one field
          in:         # X. or in several at once
            - <field>
            - <field>
      # debug related actions:
      - caps # make the message title uppercase
      - debug_print # debug print the entire contents of the entry

sink:
  ... # same as process: sink. Just appends itself to the process list. This is useful when the process list is set in a template and thus can't be overriden
```

