use crate::{Method, Request, URL};

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Get](enum.Method.html).
pub fn get<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Get, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Head](enum.Method.html).
pub fn head<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Head, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Post](enum.Method.html).
pub fn post<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Post, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Put](enum.Method.html).
pub fn put<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Put, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Delete](enum.Method.html).
pub fn delete<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Delete, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Connect](enum.Method.html).
pub fn connect<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Connect, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Options](enum.Method.html).
pub fn options<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Options, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Trace](enum.Method.html).
pub fn trace<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Trace, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Patch](enum.Method.html).
pub fn patch<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Patch, url)
}
