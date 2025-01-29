# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.13.2] - 2025-01-29
### Fixed
- Reverted a part of 2.13.1, accidentally removed some code that wasn't actually
  dead code.

## [2.13.1] - 2025-01-29
### Fixed
- Usage of an openssl-probe function that's deprecated due to safety issues. See
  [rustsec/advisory-db#2209](https://github.com/rustsec/advisory-db/pull/2209)
  for further info.

## [2.13.0] - 2024-12-04
### Changed
- The `https-rustls-probe` feature no longer brings in the `webpki-roots` and
  `rustls-webpki` crates. Thanks for the report, @polarathene!
  ([#111](https://github.com/neonmoe/minreq/issues/111))

### Fixed
- Cleaned up an unnecessary `format!()` in `Connection::connect`. Thanks for the
  PR, @melotic! ([#112](https://github.com/neonmoe/minreq/pull/112))
- Fixed some msrv and lint issues introduced by libc and clippy updates
  respectively.

## [2.12.0] - 2024-07-16
### Added
- Request::with_headers, to allow passing in many headers at a
  time. Thanks for the idea and PR, @rawhuul!
  ([#110](https://github.com/neonmoe/minreq/pull/110))

## [2.11.2] - 2024-04-26
### Fixed
- The dev dependency tiny_http's version up to 0.12. Thanks for the
  PR, @davide125! ([#107](https://github.com/neonmoe/minreq/pull/107))

## [2.11.1] - 2024-02-04
### Fixed
- Unnecessary buffering causing performance problems. Thanks for the
  PRs, @mrkline! ([#102](https://github.com/neonmoe/minreq/pull/102),
  [#103](https://github.com/neonmoe/minreq/pull/103))
- Connections failing if the first resolved address fails to connect
  (even if there's more to try). Thanks for the PR, @darosior!
  ([#106](https://github.com/neonmoe/minreq/pull/106))

## [2.11.0] - 2023-10-17
### Changed
- Removed upper bounds on the `serde_json`, `log` and `chrono`
  dependencies (dev-dependency in the case of `chrono`). If you were
  depending on minreq compiling with the MSRV compiler without any
  issues, check out the MSRV section in the readme, it's been updated
  with additional instructions. Thanks for the report, @RCasatta!
  ([#99](https://github.com/neonmoe/minreq/issues/99))

## [2.10.0] - 2023-09-05
### Fixed
- Fragment handling, once again. Turns out you're not supposed to include
  fragments in the request. This may break usage with servers that are written
  with the wrong assumptions. Thanks for the report, @rawhuul!
  ([#100](https://github.com/neonmoe/minreq/issues/100))

### Added
- `Response::url` and `ResponseLazy::url` fields, to contain the final URL after
  redirects and fragment replacement semantics.

## [2.9.1] - 2023-08-28
### Changed
- Loosened the rustls version requirement from 0.21.6 to 0.21.1.

## [2.9.0] - 2023-08-24
### Changed
- From webpki to rustls-webpki. Thanks for the heads-up about webpki not
  being maintained, @RCasatta!
  ([#98](https://github.com/neonmoe/minreq/issues/98))
- Updated rustls and webpki-roots to their most recent versions.
- Maximum versions for the following dependencies to keep minreq compiling on
  Rust 1.48:
  - serde_json (`>=1.0.0, <1.0.101`)
  - log (`>=0.4.0, <0.4.19`)
  - chrono (dev-dependency, `>=0.4.0, <0.4.24`)

### Fixed
- Errors when using an IP address as the host with HTTPS (tested with
  <https://8.8.8.8>). ([#34](https://github.com/neonmoe/minreq/issues/34))

## [2.8.1] - 2023-05-20
### Fixed
- Proxy strings with the protocol included not working. Thanks for the report,
  @tkkcc! ([#95](https://github.com/neonmoe/minreq/issues/95))

## [2.8.0] - 2023-05-13
### Added
- Default proxy from environment variables when the `proxy` feature is
  enabled, based on what curl does. Thanks for the PR, @krypt0nn!
  ([#94](https://github.com/neonmoe/minreq/pull/94))

## [2.7.0] - 2023-03-19
### Changed
- From lazy_static to once_cell for library internals. Thanks for the PR,
  @alpha-tango-kilo! ([#80](https://github.com/neonmoe/minreq/pull/80))

### Added
- A Read impl for ResponseLazy. Thanks for the PR, @Luro02!
  ([#81](https://github.com/neonmoe/minreq/pull/81))
- Building with `--all-features`, with the `send_https` function defaulting to
  the rustls-based implementation. Thanks for the PR, @tcharding!
  ([#89](https://github.com/neonmoe/minreq/pull/89))
- An explicit minimum supported rust version policy. The MSRV for versions 2.x
  is 1.48. Thanks for the suggestion and PR, @tcharding!
  ([#90](https://github.com/neonmoe/minreq/pull/90))
- Performance improvements, test fixes, CI updates.

## [2.6.0] - 2022-02-23
### Changed
- The error returned when the request url does not start with
  `https://` or `http://` now is now a slightly different IoError,
  with a clearer message. This will be changed to a proper
  minreq-specific error in 3.0, but for now it's an IoError to avoid
  breaking the Error type.

### Added
- The `urlencoding` feature for automatically percent-encoding
  urls. Thanks for the idea and PR, @alpha-tango-kilo!
  ([#67](https://github.com/neonmoe/minreq/issues/67),
  [#68](https://github.com/neonmoe/minreq/pull/68))

## [2.5.1] - 2022-01-07
### Fixed
- GitHub API requests without User-Agent returning an IoError. Thanks
  for the report, @tech-ticks!
  ([#66](https://github.com/neonmoe/minreq/issues/66))

## [2.5.0] - 2022-01-06
### Fixed
- Returning the wrong status code when the response was missing a
  status phrase. Thanks for the PR, @richarddd!
  ([#64](https://github.com/neonmoe/minreq/issues/64))
- Non-lazy requests crashing if the request had a very big
  Content-Length header. Thanks for the report, @Shnatsel!
  ([#63](https://github.com/neonmoe/minreq/issues/63))

## [2.4.2] - 2021-06-11
### Fixed
- A regression in 2.4.1 where the port is no longer included in the
  `Host`, even if it's a non-standard port. Now the port is always
  included if it's in the request URL, and omitted if the port is
  implied. Thanks for the report, @ollpu!
  ([#61](https://github.com/neonmoe/minreq/issues/61))

## [2.4.1] - 2021-06-05
### Fixed
- The port is no longer included in the `Host` header when sending
  requests, and port handling was cleaned up overall. This fixes
  issues with infinite redirections and https handshakes for some
  websites. Thanks to @Shnatsel for reporting the issues, and @joeried
  for debugging and figuring out the root cause of these problems!
  ([#48](https://github.com/neonmoe/minreq/issues/48),
  [#49](https://github.com/neonmoe/minreq/issues/49))

## [2.4.0] - 2021-05-27
### Added
- `Request::with_param` for more ergonomic query parameter
  usage. Thanks for the PR, @sjvignesh!
  ([#54](https://github.com/neonmoe/minreq/pull/54))
- `Request::with_max_headers_size` and
  `Request::with_max_status_line_length` for avoiding DoS when the
  server sends large headers or status lines. Thanks for the report,
  @Shnatsel! ([#55](https://github.com/neonmoe/minreq/issues/55))
- Support for the `rustls-native-certs` crate via a new
  `https-rustls-probe` feature. Thanks for the PR, @joeried!
  ([#59](https://github.com/neonmoe/minreq/pull/59))

### Fixed
- Chunk length handling for some servers with slightly off-spec chunk
  lengths. Thanks for the report, @Shnatsel!
  ([#50](https://github.com/neonmoe/minreq/issues/50))
- Timeouts not always being properly enforced. Thanks for the report,
  @Shnatsel! ([#52](https://github.com/neonmoe/minreq/issues/52))

## [2.3.1] - 2021-02-10
### Fixed
- Removed some leftover printlns from the redirection update in 2.3.0
  and ensured there's no printlns in the library anymore. Thanks for
  reporting the issue @Shnatsel!
  [#45](https://github.com/neonmoe/minreq/issues/45)
- Fixed the timeout not being respected during the initial TCP
  connect. Thanks for the report and fix @KarthikNedunchezhiyan!
  [#46](https://github.com/neonmoe/minreq/issues/46),
  [#47](https://github.com/neonmoe/minreq/pull/47)

## [2.3.0] - 2021-01-04
### Changed
- **Breaking (sort of):** the redirection code was improved to match
  [RFC 7231 section
  7.1.2](https://tools.ietf.org/html/rfc7231#section-7.1.2), which
  could subtly break some programs relying on very specific redirects,
  which is why this should be investigated if you come across weird
  behaviour after updating. No API changes though, so only a minor
  version bump. The following two points are now fixed when
  redirecting:
  - Fragments, the bit after a #-character in the url. If the
    redirecting url has a fragment, and the one in `Location` does
    not, the original fragment should be included in the new url. If
    `Location` does have a fragment, it should override the one in the
    redirecting url.
  - Relative urls. Minreq now properly redirects when `Location` is
    relative, e.g. `/Foo.html` instead of
    `https://example.com/Foo.html`. Thanks, @fjt523!

### Fixed
- The `Content-Length: 0` header is now inserted into requests that
  should have it. Thanks, @KarthikNedunchezhiyan!
- Status line parsing is now fixed, so "400 Bad Request" is not parsed
  as "400 Bad". Thanks, @KarthikNedunchezhiyan!

### Added
- M1 Mac support by bumping the ring dependency. Thanks, @ryanmcgrath!

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
