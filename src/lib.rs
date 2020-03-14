//! # Minreq
//! Simple, minimal-dependency HTTP client.
//! The library has a very minimal API, so you'll probably know
//! everything you need to after reading a few examples.
//!
//! # Additional features
//!
//! Since the crate is supposed to be minimal in terms of
//! dependencies, there are no default features, and optional
//! functionality can be enabled by specifying features for `minreq`
//! dependency in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! minreq = { version = "*", features = ["https", "json-using-serde", "punycode"] }
//! ```
//!
//! Below is the list of all available features.
//!
//! ## `https`
//!
//! This feature uses the (very good)
//! [`rustls`](https://crates.io/crates/rustls) crate to secure the
//! connection when needed. Note that if this feature is not enabled
//! (and it is not by default), requests to urls that start with
//! `https://` will fail and return a
//! [`HttpsFeatureNotEnabled`](enum.Error.html#variant.HttpsFeatureNotEnabled)
//! error.
//!
//! ## `json-using-serde`
//!
//! This feature allows both serialize and deserialize JSON payload
//! using the [`serde_json`](https://crates.io/crates/serde_json)
//! crate.
//!
//! [`Request`](struct.Request.html) and
//! [`Response`](struct.Response.html) expose
//! [`with_json()`](struct.Request.html#method.with_json) and
//! [`json()`](struct.Response.html#method.json) for constructing the
//! struct from JSON and extracting the JSON body out, respectively.
//!
//! ## `punycode`
//!
//! This feature enables requests to non-ascii domains: the
//! [`punycode`](https://crates.io/crates/punycode) crate is used to
//! convert the non-ascii parts into their punycode representations
//! before making the request. If you try to make a request to 㯙㯜㯙
//! 㯟.net or i❤.ws for example, with this feature disabled (as it is
//! by default), your request will fail with a
//! [`PunycodeFeatureNotEnabled`](enum.Error.html#variant.PunycodeFeatureNotEnabled)
//! error.
//!
//! # Examples
//!
//! ## Get
//!
//! This is a simple example of sending a GET request and printing out
//! the response's body, status code, and reason phrase. The `?` are
//! needed because the server could return invalid UTF-8 in the body,
//! or something could go wrong during the download.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = minreq::get("http://httpbin.org/ip").send()?;
//! assert!(response.as_str()?.contains("\"origin\":"));
//! assert_eq!(response.status_code, 200);
//! assert_eq!(response.reason_phrase, "OK");
//! # Ok(()) }
//! ```
//!
//! Note: you could change the `get` function to `head` or `put` or
//! any other HTTP request method: the api is the same for all of
//! them, it just changes what is sent to the server.
//!
//! ## Body (sending)
//!
//! To include a body, add `with_body("<body contents>")` before
//! `send()`.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = minreq::post("http://httpbin.org/post")
//!     .with_body("Foobar")
//!     .send()?;
//! assert!(response.as_str()?.contains("Foobar"));
//! # Ok(()) }
//! ```
//!
//! ## Headers (sending)
//!
//! To add a header, add `with_header("Key", "Value")` before
//! `send()`.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = minreq::get("http://httpbin.org/headers")
//!     .with_header("Accept", "text/plain")
//!     .with_header("X-Best-Mon", "Sylveon")
//!     .send()?;
//! let body_str = response.as_str()?;
//! assert!(body_str.contains("\"Accept\": \"text/plain\""));
//! assert!(body_str.contains("\"X-Best-Mon\": \"Sylveon\""));
//! # Ok(()) }
//! ```
//!
//! ## Headers (receiving)
//!
//! Reading the headers sent by the servers is done via the
//! [`headers`](struct.Response.html#structfield.headers) field of the
//! [`Response`](struct.Response.html). Note: the header field names
//! (that is, the *keys* of the `HashMap`) are all lowercase: this is
//! because the names are case-insensitive according to the spec, and
//! this unifies the casings for easier `get()`ing.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = minreq::get("http://httpbin.org/ip").send()?;
//! assert_eq!(response.headers.get("content-type").unwrap(), "application/json");
//! # Ok(()) }
//! ```
//!
//! ## Timeouts
//! To avoid timing out, or limit the request's response time, use
//! `with_timeout(n)` before `send()`. The given value is in seconds.
//!
//! NOTE: There is no timeout by default.
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = minreq::post("http://httpbin.org/delay/6")
//!     .with_timeout(10)
//!     .send()?;
//! println!("{}", response.as_str()?);
//! # Ok(()) }
//! ```
//!
//! ## Proxy
//! To use a proxy server, simply provide create a `Proxy` instance
//! and use `.with_proxy()` on your request.
//!
//! Supported proxy formats are `host:port` and `user:password@proxy:host`.
//! Only HTTP CONNECT proxies are supported at this time.
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let proxy = minreq::Proxy::new("localhost:8080")?;
//! let response = minreq::post("http://httpbin.org/delay/1")
//!     .with_proxy(proxy)
//!     .send()?;
//! println!("{}", response.as_str()?);
//! # Ok(()) }
//! ```
//!
//!
//! # Timeouts
//! By default, a request has no timeout.  You can change this in two ways:
//! - Use [`with_timeout`](struct.Request.html#method.with_timeout) on
//!   your request to set the timeout per-request like so:
//!   ```
//!   minreq::get("/").with_timeout(8).send();
//!   ```
//! - Set the environment variable `MINREQ_TIMEOUT` to the desired
//!   amount of seconds until timeout. Ie. if you have a program called
//!   `foo` that uses minreq, and you want all the requests made by that
//!   program to timeout in 8 seconds, you launch the program like so:
//!   ```text,ignore
//!   $ MINREQ_TIMEOUT=8 ./foo
//!   ```
//!   Or add the following somewhere before the requests in the code.
//!   ```
//!   std::env::set_var("MINREQ_TIMEOUT", "8");
//!   ```
//! If the timeout is set with `with_timeout`, the environment
//! variable will be ignored.

#![deny(missing_docs)]

#[cfg(feature = "https")]
extern crate rustls;
#[cfg(feature = "json-using-serde")]
extern crate serde;
#[cfg(feature = "json-using-serde")]
extern crate serde_json;
#[cfg(feature = "https")]
extern crate webpki;
#[cfg(feature = "https")]
extern crate webpki_roots;

mod connection;
mod error;
mod proxy;
mod request;
mod response;

pub use error::*;
pub use proxy::*;
pub use request::*;
pub use response::*;
