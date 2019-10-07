# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- API for loading the HTTP response body through an iterator, allowing
  for processing of the data during the download.
- Error type for all the errors that this crate can run into for
  easier `?` usage and better debuggability.
- Punycode support for non-ascii hostnames via the `punycode` feature.

### Changed
- Update dependencies.
- Restructure the `Response` struct in a major, breaking way.
- Clean up the crate internals overall. **Note**: This might cause
  instability, if you're very concerned about stability, please hold
  off upgrading for a while.
- Remove `panic!` when trying to make an `https://` request without
  the `https` feature. The request will now return an error
  instead. The library should not panic anymore.
- Audit the remaining `unwrap()`s from library code, none of them 
  should actually ever cause a panic now.

### Fixed
- Test on Windows by changing the ip in tests from `0.0.0.0` to
  `localhost`.

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
