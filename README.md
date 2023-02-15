# minreq
[![Crates.io](https://img.shields.io/crates/d/minreq.svg)](https://crates.io/crates/minreq)
[![Documentation](https://docs.rs/minreq/badge.svg)](https://docs.rs/minreq)
[![CI](	https://img.shields.io/travis/neonmoe/minreq.svg)](https://travis-ci.org/neonmoe/minreq)

Simple, minimal-dependency HTTP client. Optional features for json
responses (`json-using-serde`), unicode domains (`punycode`), http
proxies (`proxy`), and https with various TLS implementations
(`https-rustls`, `https-rustls-probe`, `https-bundled`,
`https-bundled-probe`,`https-native`, and `https` which is an alias
for `https-rustls`).

Without any optional features, my casual testing indicates about 100
KB additional executable size for stripped release builds using this
crate. Compiled with rustc 1.45.2, `println!("Hello, World!");` is 239
KB on my machine, where the [hello](examples/hello.rs) example is 347
KB. Both are pure Rust, so aside from `libc`, everything is statically
linked.

Note: some of the dependencies of this crate (especially `serde` and
the various `https` libraries) are a lot more complicated than this
library, and their impact on executable size reflects that.

## [Documentation](https://docs.rs/minreq)

## Planned for 3.0.0

This is a list of features I'll implement once it gets long enough, or
a severe enough issue is found that there's good reason to make a
major version bump.

- Change the response/request structs to allow multiple headers with
  the same name.
- Set sane defaults for maximum header size and status line
  length. The ability to add maximums was added in response to
  [#55](https://github.com/neonmoe/minreq/issues/55), but defaults for
  the limits is a breaking change.
- Clearer error when making a request to an url that does not start
  with `http://` or `https://`.
- Non-exhaustive error type?

## Minimum Supported Rust Version (MSRV)

We use an MSRV per major release, i.e., with a new major release we
reserve the right to change the MSRV.

The current major version (v2) of this library should always compile
with any combination of features **excluding TLS** on **Rust 1.48**.

## License
This crate is distributed under the terms of the [ISC license](COPYING.md).
