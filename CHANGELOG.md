# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Loading HTTP responses through an iterator, allowing processing the
  data during the download.

### Changed
- Updated dependencies
- Restructure the `Response` struct in a major, breaking way.

### Fixed
- Changed ip in tests from `0.0.0.0` to `localhost` to fix them
  running on Windows.

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
