# minreq
[![Crates.io](https://img.shields.io/crates/d/minreq.svg)](https://crates.io/crates/minreq)
[![Documentation](https://docs.rs/minreq/badge.svg)](https://docs.rs/minreq)
[![CI](	https://img.shields.io/travis/neonmoe/minreq.svg)](https://travis-ci.org/neonmoe/minreq)

Simple, minimal-dependency HTTP client. Optional features for https
(`https`), json via Serde (`json-using-serde`), and unicode domains
(`punycode`).

Without any optional features, my casual testing indicates about 100
KB additional executable size for stripped release builds using this
crate.

## [Documentation](https://docs.rs/minreq)

## License
This crate is distributed under the terms of the [ISC license](COPYING.md).
