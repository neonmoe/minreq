use crate::{Method, Request, URL};

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
pub fn create_request<T: Into<URL>>(method: Method, url: T) -> Request {
    Request::new(method, url.into())
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Get](enum.Method.html).
pub fn get<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Get, url)
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Head](enum.Method.html).
pub fn head<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Head, url)
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Post](enum.Method.html).
pub fn post<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Post, url)
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Put](enum.Method.html).
pub fn put<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Put, url)
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Delete](enum.Method.html).
pub fn delete<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Delete, url)
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Connect](enum.Method.html).
pub fn connect<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Connect, url)
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Options](enum.Method.html).
pub fn options<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Options, url)
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Trace](enum.Method.html).
pub fn trace<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Trace, url)
}

/// Alias for [create_request](fn.create_request.html) with `method` set to
/// [Method::Patch](enum.Method.html).
pub fn patch<T: Into<URL>>(url: T) -> Request {
    create_request(Method::Patch, url)
}
