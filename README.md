# minreq
[![Crates.io](https://img.shields.io/crates/d/minreq.svg)](https://crates.io/crates/minreq)
[![Documentation](https://docs.rs/minreq/badge.svg)](https://docs.rs/minreq)
![Unit tests](https://github.com/neonmoe/minreq/actions/workflows/unit-tests.yml/badge.svg)
![MSRV](https://github.com/neonmoe/minreq/actions/workflows/msrv.yml/badge.svg)

Simple, minimal-dependency HTTP client. Optional features for json
responses (`json-using-serde`), unicode domains (`punycode`), http
proxies (`proxy`), and https with various TLS implementations
(`https-rustls`, `https-rustls-probe`, `https-bundled`,
`https-bundled-probe`,`https-native`, and `https` which is an alias
for `https-rustls`).

Without any optional features, my casual testing indicates about 148
KB additional executable size for stripped release builds using this
crate. Compiled with rustc 1.94.0, `println!("Hello, World!");` is 343
KB on my machine, where the [hello](examples/hello.rs) example is 491
KB. Both are pure Rust, so aside from `libc`, everything is statically
linked.

Note: some of the dependencies of this crate (especially `serde` and
the various `https` libraries) are a lot more complicated than this
library, and their impact on executable size reflects that.

## Documentation

Build your own with `cargo doc --all-features`, or browse the online
documentation at [docs.rs/minreq](https://docs.rs/minreq).

## Minimum Supported Rust Version (MSRV)

This project has a stable MSRV policy per major release.

The current major version (v3) of this library is intended to compile on the
version of Rust found in Debian oldstable when a particular version of minreq is
released. At the time of writing, it is **Rust 1.63** from Debian bookworm.

The rationale for this policy is to not need to make a major version bump just
for an MSRV bump in the future, as having 1.48 set in stone for minreq v2 forced
a major version bump due to a tough incompatibility issue with a new version of
rustls (even without the rustls features enabled for MSRV builds, see
[#123](https://github.com/neonmoe/minreq/issues/123) and
[#124](https://github.com/neonmoe/minreq/pull/124)). Debian oldstable is the
target, because buildling on an old-ish distro might be useful for e.g. avoiding
depending on a new version of glibc. Distributing Linux binaries is so fun.

Any optional features might come with their own (higher) MSRVs, this policy only
applies to minreq without any features enabled. Check the MSRV CI job for
features that happen to currently work at the MSRV (they will be dropped if they
stop compiling).

Major version 2 of minreq had an MSRV of 1.48 (except for https features).

## License
This crate is distributed under the terms of the [ISC license](COPYING.md).

## Planned for 3.0.0

This is a list of features I'll implement once it gets long enough, or
a severe enough issue is found that there's good reason to make a
major version bump.

- Change the response/request structs to allow multiple headers with
  the same name.

### Potential ideas

Just thinking out loud, might not end up doing some or all of these.

- Would be good if the crate got smaller with 3.0, not bigger. Maybe
  there's something to cut, something to optimize?
