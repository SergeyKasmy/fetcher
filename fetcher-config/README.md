# fetcher-config

The config format of [`fetcher-core`][fetcher-core] and [`fetcher`][fetcher]. Specified externally on purpose to separate it and allow updating it without updating/changing [`fetcher-core`][fetcher-core].
It's modal enough not to be tied to [`fetcher`][fetcher] and can be used to write a different frontend/UX. Maybe some day I'll make a GTK GUI for it...

[fetcher-core]: https://docs.rs/fetcher-core/latest/fetcher_core/
[fetcher]: https://docs.rs/crate/fetcher/latest
