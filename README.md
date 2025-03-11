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
- Change default proxy port to 1080 (from 8080). Curl uses 1080, so it's a sane
  default.
- Bump MSRV enough to compile the latest versions of all dependencies, and add
  the `rust-version` (at least 1.56) and `edition` (at least 2021) fields to
  Cargo.toml.

## Minimum Supported Rust Version (MSRV)

If you don't care about the MSRV, you can ignore this section
entirely, including the commands instructed.

We use an MSRV per major release, i.e., with a new major release we
reserve the right to change the MSRV.

The current major version (v2) of this library should always compile with any
combination of features excluding the TLS and urlencoding features on **Rust
1.48**. This is because those dependencies themselves have a higher MSRV.

That said, the crate does still require forcing some dependencies to
lower-than-latest versions to actually compile with the older
compiler, as these dependencies have upped their MSRV in a patch
version. This can be achieved with the following (these just update
your Cargo.lock):

```sh
cargo update --package=log --precise=0.4.18
cargo update --package=httpdate --precise=1.0.2
cargo update --package=serde_json --precise=1.0.100
cargo update --package=chrono --precise=0.4.23
cargo update --package=num-traits --precise=0.2.18
cargo update --package=tempfile --precise=3.17.1
cargo update --package=libc --precise=0.2.163
# This again, for some reason.
cargo update --package=httpdate --precise=1.0.2
```

## License
This crate is distributed under the terms of the [ISC license](COPYING.md).
