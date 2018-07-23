# mbox-prime
Simple, read-only mailbox reader TUI written in Rust, with vi-like controls.

Primarily a personal project to understand more about the mbox format and 
how many ways you can encode an email.


# Controls
`mbox-prime` reads a single uncompressed mbox file:
```
mbox-prime opensuse-factory-2018-06.mbox
```

Emails are displayed in the left pane, and the current selected email in the
right.  Focus is switched between the two panes using Tab.

`j` and `k` move up and down, as do `Pgup` and `PgDn`. To follow email threads,
`h` and `l` jump to the selected email's parent or child, respectively. If a
message has multiple replies, `n` and `N` jump to the next and previous 
"sibling" reply - replies to the same parent.


# Status
Generally does a great job with plain/text, quoted-printable, and multipart 
messages.

The following could use improvement:
[ ] base64 encoded emails mixed with plain text - base64 is all-or-nothing
[ ] user interface isn't quite colorful enough
[ ] Both panes could use scrollbar indicators
[ ] support reading compressed files


