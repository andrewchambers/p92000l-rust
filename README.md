# P92000L-rust

A rust library for the 9P2000.L protocol. Note, the name starts with P9 instead of 9P because rust forbids identifiers that begin with numbers.

This library is primarily developed to facilitate user space rust 9P2000.L clients
interacting with the [distributed io daemon](https://github.com/chaos/diod).
Rust programs should be able to connect to 9P2000.L servers directly, or
alternatively via a yet to be implemented fuse client.

A secondary stretch goal of this library is to provide a high performance 9P2000.L
server in a memory safe language.

In other words, the library aims to:

- Implement 9p2000.L.
- Provide an interface for 9p2000.L clients that is usable from multiple threads concurrently.
- Provide interfaces for implementing multi threaded 9p2000.L servers.

In general the library will:

- Focus on synchronous (not async) rust.
- Focus on high performance and being low overhead.
