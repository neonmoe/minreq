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
/// it is implied in the name, and the body is as optional as it is
/// on [Wikipedia](https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol#Summary_table).
///
/// # Examples
///
/// ### Using `minreq::send`
///
/// ```no_run
/// use minreq::Method;
///
/// // This application prints out your public IP. (Or an error.)
/// match minreq::send(Method::Get, "https://httpbin.org/ip", None) {
///     Ok(response) => println!("Your public IP: {}", response.body),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
///
/// ### Using the aliases ie. how you'll actually probably use this crate
///
/// ```no_run
/// // This is the same as above, except less elaborate, and more panic-y.
/// if let Ok(response) = minreq::get("https://httpbin.org/ip", None) {
///     println!("Your public IP: {}", response.body);
/// }
/// ```
pub fn send<T: Into<URL>>(method: Method, url: T, body: Option<String>) -> Result<Response, Error> {
    let request = Request::new(method, url.into(), body);
    let connection = Connection::new(request);
    connection.send()
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Get](enum.Method.html).
pub fn get<T: Into<URL>>(url: T, body: Option<String>) -> Result<Response, Error> {
    send(Method::Get, url, body)
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Head](enum.Method.html).
pub fn head<T: Into<URL>>(url: T) -> Result<Response, Error> {
    send(Method::Head, url, None)
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Post](enum.Method.html).
pub fn post<T: Into<URL>>(url: T, body: String) -> Result<Response, Error> {
    send(Method::Post, url, Some(body))
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Put](enum.Method.html).
pub fn put<T: Into<URL>>(url: T, body: String) -> Result<Response, Error> {
    send(Method::Put, url, Some(body))
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Delete](enum.Method.html).
pub fn delete<T: Into<URL>>(url: T) -> Result<Response, Error> {
    send(Method::Delete, url, None)
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Connect](enum.Method.html).
pub fn connect<T: Into<URL>>(url: T, body: String) -> Result<Response, Error> {
    send(Method::Connect, url, Some(body))
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Options](enum.Method.html).
pub fn options<T: Into<URL>>(url: T, body: Option<String>) -> Result<Response, Error> {
    send(Method::Options, url, body)
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Trace](enum.Method.html).
pub fn trace<T: Into<URL>>(url: T) -> Result<Response, Error> {
    send(Method::Trace, url, None)
}

/// Alias for [send](fn.send.html) with `method` set to [Method::Patch](enum.Method.html).
pub fn patch<T: Into<URL>>(url: T, body: String) -> Result<Response, Error> {
    send(Method::Patch, url, Some(body))
}
