# P92000L

A rust library for 9P2000L.

The goals of the library:

- Only support 9P2000L.
- Provide an interface for single threaded 9p2000L servers.
- Provide an interface for multi threaded 9p2000L servers.
- Provide an interface for 9p2000L clients that is usable from multiple threads.
- Provide support for as many platforms as possible.
- Focus on synchronous (not async) rust.
- Focus on a small dependency footprint.
