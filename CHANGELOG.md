# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.1] - 2020-08-22
### Fixed
- Some documentation which has been long due for an update. I just
  always forget when writing an actual update. No code changes!

## [2.2.0] - 2020-06-18
### Added
- Support for `native-tls` and `openssl-sys` via new features, in
  addition to `rustls`. Thanks to @dubiousjim!

## [2.1.1] - 2020-05-01
### Fixed
- Handling of status codes 204 and 304. Thanks to @Mubelotix!

## [2.1.0] - 2020-03-14
### Added
- Proxy support via the `proxy` feature. Thanks to @rustysec!

## [2.0.3] - 2020-01-15
### Fixed
- Fixed regression in header parsing caused by 2.0.2, which was yanked.

## [2.0.2] - 2020-01-15
### Fixed
- Fixed a panic when sending a request to an invalid domain
  via https.
- Fixed a panic when parsing headers that have >1 byte
  unicode characters right after the ":" in the response.

## [2.0.1] - 2020-01-11
### Fixed
- Made timeouts work as described in the documentation.
  Fixed issue #22.

## [2.0.0] - 2019-11-23
### Added
- API for loading the HTTP response body through an iterator, allowing
  for processing of the data during the download.
  - See the `ResponseLazy` documentation for more information.
- Error type for all the errors that this crate can run into for
  easier `?` usage and better debuggability.
- Punycode support for non-ascii hostnames via the `punycode` feature.
- Trailer header support.
- Examples [`hello`](examples/hello.rs),
  [`iterator`](examples/iterator.rs), and [`json`](examples/json.rs).

### Changed
- **Breaking, will cause problems not detectable by the compiler:**
  Response headers' field names are now in lowercase, as they are
  case-insensitive and this makes getting header values easier. The
  values are unaffected. So if your code has
  `response.headers.get("Content-Type")`, you have to change it to
  `response.headers.get("content-type")`, or it will not return what
  you want.
- **Breaking**: Restructure the `Response` struct:
  - Removed `bytes` and `body_bytes`.
  - Added `as_bytes()`, `into_bytes()`, and `as_str()` in their place.
- **Breaking**: Changed the `with_body` parameter type to
  `Into<Vec<u8>>` from `Into<String>`.
  - `String`s implement `Into<Vec<u8>>`, so this shouldn't cause any
    problems, unless you're using some interesting types that
    implement `Into<String>` but not `Into<Vec<u8>>`.
- Clean up the crate internals overall. **Note**: This might cause
  instability, if you're very concerned about stability, please hold
  off upgrading for a while.
- Remove `panic!` when trying to make an `https://` request without
  the `https` feature. The request will now return an error
  instead. The library should not panic anymore.
- Audit the remaining `unwrap()`s from library code, none of them
  should actually ever cause a panic now.

### Removed
- `create_request` in favor of just using `Response::new`.

## [1.4.1] - 2019-10-13
### Changed
- Updated dependencies.

### Fixed
- Tests on Windows by changing the ip in tests from `0.0.0.0` to
  `localhost`.
- Reuse `rustls::ClientConfig` between requests.
- `Content-Length` and `Transfer-Encoding` detection failing because
  of case-sensitiveness.

## [1.4.0] - 2019-07-13
### Added
- `json-using-serde` feature.

## [1.3.0] - 2019-06-04
### Added
- The `body_bytes` field to Response, containing the body in raw
  bytes.

### Fixed
- Some clippy warnings.
- Panic when getting a non-UTF-8 response, instead setting the `body`
  string to an empty string, for now.

## [1.2.1] - 2019-05-24
### Fixed
- HTTP response body handling.

## [1.2.0] - 2019-05-23
### Added
- Support for the HTTP status codes 301, 302, 303, and 307.

### Fixed
- Less .clones()s.

## [1.1.2] - 2019-04-14
### Fixed
- Fix response handling when `Transfer-Encoding` is `chunked`.

## [1.1.1] - 2019-03-28
### Changed
- Moved to 2018 edition.

### Fixed
- HEAD requests and ones that receive a 1xx, 204, or 304 status code
  as a response.

## [1.1.0] - 2019-03-24
### Changed
- Timeout made optional.
- Updated dependencies.

### Fixed
- Improved performance for HTTP (not HTTPS) requests.
