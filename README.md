# minreq
[![Crates.io](https://img.shields.io/crates/d/minreq.svg)](https://crates.io/crates/minreq)
[![Documentation](https://docs.rs/minreq/badge.svg)](https://docs.rs/minreq)
[![CI](	https://img.shields.io/travis/neonmoe/minreq.svg)](https://travis-ci.org/neonmoe/minreq)

Simple, minimal-dependency HTTP client. Optional features for json
responses (`json-using-serde`), unicode domains (`punycode`), http
proxies (`proxy`), and https with various TLS implementations
(`https-rustls`, `https-bundled`, `https-bundled-probe`,
`https-native`, and `https` which is an alias for `https-rustls`).

Without any optional features, my casual testing indicates about 100
KB additional executable size for stripped release builds using this
crate.

## [Documentation](https://docs.rs/minreq)

## License
This crate is distributed under the terms of the [ISC license](COPYING.md).
