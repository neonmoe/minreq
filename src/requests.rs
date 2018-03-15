use std::io::Error;
use http::{Method, Request, Response, URL};
use connection::Connection;

/// Sends a request to `url` with `method`, returns the response or
/// an [`Error`](https://doc.rust-lang.org/std/io/struct.Error.html).
///
/// In most cases it is recommended to use one of the aliases of this
/// function: [`get`](fn.get.html), [`head`](fn.head.html),
/// [`post`](fn.post.html), [`put`](fn.put.html),
/// [`delete`](fn.delete.html), [`trace`](fn.trace.html),
/// [`options`](fn.options.html), [`connect`](fn.connect.html),
/// [`patch`](fn.patch.html). They omit the `method` parameter, since
/// it is implied in the name, and the body is as optional as it is on
/// [Wikipedia](https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol#Summary_table).
///
/// The timeout of the created request is 5 seconds by default. You
/// can change this in two ways:
/// - Use this function (`create_connection`) and call
///   [`with_timeout`](struct.Connection.html#method.with_timeout)
///   on it to set the timeout per-request.
/// - Set the environment variable `MINREQ_TIMEOUT` to the desired
///   amount of seconds until timeout. Ie. if you have a program called
///   `foo` that uses minreq, and you want all the requests made by that
///   program to timeout in 8 seconds, you launch the program like so:
///   ```text,ignore
///   $ MINREQ_TIMEOUT=8 ./foo
///   ```
///   Or add the following somewhere before the requests in the code.
///   ```
///   use std::env;
///
///   env::set_var("MINREQ_TIMEOUT", "8");
///   ```
///
/// # Examples
///
/// ### Using `minreq::send`
///
/// ```no_run
/// use minreq::Method;
///
/// // This application prints out your public IP. (Or an error.)
/// match minreq::create_connection(Method::Get, "https://httpbin.org/ip", None).send() {
///     Ok(response) => println!("Your public IP: {}", response.body),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
///
/// ### Using the aliases ie. how you'll actually probably use this crate
///
/// ```no_run
/// // This is the same as above, except less elaborate.
/// if let Ok(response) = minreq::get("https://httpbin.org/ip", None) {
///     println!("Your public IP: {}", response.body);
/// }
/// ```
pub fn create_connection<T: Into<URL>>(method: Method, url: T, body: Option<String>) -> Connection {
    let request = Request::new(method, url.into(), body);
    Connection::new(request)
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Get](enum.Method.html).
pub fn get<T: Into<URL>>(url: T, body: Option<String>) -> Result<Response, Error> {
    create_connection(Method::Get, url, body).send()
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Head](enum.Method.html).
pub fn head<T: Into<URL>>(url: T) -> Result<Response, Error> {
    create_connection(Method::Head, url, None).send()
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Post](enum.Method.html).
pub fn post<T: Into<URL>>(url: T, body: String) -> Result<Response, Error> {
    create_connection(Method::Post, url, Some(body)).send()
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Put](enum.Method.html).
pub fn put<T: Into<URL>>(url: T, body: String) -> Result<Response, Error> {
    create_connection(Method::Put, url, Some(body)).send()
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Delete](enum.Method.html).
pub fn delete<T: Into<URL>>(url: T) -> Result<Response, Error> {
    create_connection(Method::Delete, url, None).send()
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Connect](enum.Method.html).
pub fn connect<T: Into<URL>>(url: T, body: String) -> Result<Response, Error> {
    create_connection(Method::Connect, url, Some(body)).send()
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Options](enum.Method.html).
pub fn options<T: Into<URL>>(url: T, body: Option<String>) -> Result<Response, Error> {
    create_connection(Method::Options, url, body).send()
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Trace](enum.Method.html).
pub fn trace<T: Into<URL>>(url: T) -> Result<Response, Error> {
    create_connection(Method::Trace, url, None).send()
}

/// Alias for [send](fn.send.html) with `method` set to
/// [Method::Patch](enum.Method.html).
pub fn patch<T: Into<URL>>(url: T, body: String) -> Result<Response, Error> {
    create_connection(Method::Patch, url, Some(body)).send()
}
