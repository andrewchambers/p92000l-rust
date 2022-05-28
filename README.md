# P92000

A rust library for the 9P2000 and 9P2000.L protocols.

This library is primary developed to facilitate user space rust 9P2000.L clients
interacting with the [distributed io daemon](https://github.com/chaos/diod).
Rust programs should be able to connect to 9P2000.L servers directly, or
alternatively via a yet to be implemented fuse client.

A secondary stretch goal of this library is to provide a high performance 9P2000.L
server in a memory safe language.

Tertiary goals include support for other 9P variants and letting people write their own 9P servers.

In other words, the library aims to:

- Implement 9p with an initial emphasis on 9p2000.L.
- Provide an interface for 9p clients that is usable from multiple threads.
- Provide an interface for single threaded 9p servers.
- Provide an interface for multi threaded 9p servers.
- Focus on performance (zero allocation and avoid copies as much as possible).
- Focus on synchronous (not async) rust.
- Focus on a small dependency footprint.
