use std::io::Error;
use http::{Method, Request, Response, URL};
use connection::Connection;

/// Sends a GET request to `url`, returns the response or
/// an [`Error`](https://doc.rust-lang.org/std/io/struct.Error.html).
///
/// # Examples
///
/// ```no_run
/// // This application prints out your public IP. (Or an error.)
/// match minreq::get("https://httpbin.org/ip", None) {
///     Ok(response) => println!("Your public IP: {}", response.body),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
pub fn get<T: Into<URL>>(url: T, body_generic: Option<T>) -> Result<Response, Error> {
    let mut body = None;
    if let Some(body_unwrapped) = body_generic {
        body = Some(body_unwrapped.into());
    }
    let request = Request::new(Method::Get, url.into(), body);
    let connection = Connection::new(request);
    connection.send()
}

/// Sends a POST request to `url` with `body`, returns the response or
/// an [`Error`](https://doc.rust-lang.org/std/io/struct.Error.html).
///
/// # Examples
///
/// ```no_run
/// // This posts "hello" to a server, and prints out the response.
/// // (Or an error.)
/// match minreq::post("https://httpbin.org/post", Some("hello")) {
///     Ok(response) => println!("{}", response.body),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
pub fn post<T: Into<URL>>(url: T, body_generic: Option<T>) -> Result<Response, Error> {
    let mut body = None;
    if let Some(body_unwrapped) = body_generic {
        body = Some(body_unwrapped.into());
    }
    let request = Request::new(Method::Post, url.into(), body);
    let connection = Connection::new(request);
    connection.send()
}
