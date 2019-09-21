use crate::http;
use std::collections::HashMap;
use std::str;

/// An iterator for processing the response from the server during
/// the download.
///
/// To get access to this, construct your Request with
/// [`Request::with_load_later(true)`](struct.Request.html#method.with_load_later),
/// and then you can access this iterator through
/// [`Response::as_iter_mut()`](struct.Response.html#method.as_iter_mut).
///
/// # Usage
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let url = "http://example.org/";
/// let mut response = minreq::get(url).with_load_later(true).send()?;
/// for byte in response.as_iter_mut().unwrap() {
///     println!("{}", byte as char);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ResponseIter {
    loaded_bytes: Vec<u8>,
}

impl Iterator for ResponseIter {
    type Item = u8;

    /// Reads the next byte from the incoming HTTP response's body.
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

enum Body {
    Loaded(Vec<u8>),
    Loading(ResponseIter),
}

/// An HTTP response.
pub struct Response {
    /// The status code of the response, eg. 404.
    pub status_code: i32,
    /// The reason phrase of the response, eg. "Not Found".
    pub reason_phrase: String,
    /// The headers of the response.
    pub headers: HashMap<String, String>,
    /// The body of the response.
    body: Body,
}

impl Response {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Response {
        let (status_code, reason_phrase) = http::parse_status_line(&bytes);
        let (headers, body_bytes) = http::parse_http_response_content(&bytes);
        Response {
            status_code,
            reason_phrase,
            headers,
            body: Body::Loaded(body_bytes),
        }
    }

    /// Returns an iterator for reading the response as it's being
    /// loaded.
    ///
    /// If the request wasn't constructed with
    /// [`Request::with_load_later(true)`](struct.Request.html#method.with_load_later),
    /// this will always return None.
    pub fn as_iter_mut(&mut self) -> Option<&mut ResponseIter> {
        if let Body::Loading(ref mut iter) = self.body {
            Some(iter)
        } else {
            None
        }
    }

    /// Returns a `&str` constructed from the bytes returned so
    /// far. Shorthand for `std::str::from_utf8(response.as_bytes())`.
    ///
    /// Returns a `Result`, as it is possible that the returned bytes
    /// are not valid UTF-8: the message can be corrupted on the
    /// server's side, it could be still loading (`flush()` not yet
    /// called, for example), or the returned message could simply not
    /// be valid UTF-8.
    ///
    /// Usage:
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let url = "http://example.org/";
    /// let response = minreq::get(url).send()?;
    /// println!("{}", response.as_str()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        match self.body {
            Body::Loaded(ref slice) => str::from_utf8(slice),
            Body::Loading(ref iter) => str::from_utf8(&iter.loaded_bytes),
        }
    }

    /// Returns a reference to the bytes of the body.
    ///
    /// If the request was made with `.load_later()`
    ///
    /// Usage:
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let url = "http://example.org/";
    /// let response = minreq::get(url).send()?;
    /// println!("{:?}", response.as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        match self.body {
            Body::Loaded(ref slice) => slice,
            Body::Loading(ref iter) => &iter.loaded_bytes,
        }
    }

    /// Converts JSON body to a `struct` using Serde.
    ///
    /// In case compiler cannot figure out return type you might need to declare it explicitly:
    ///
    /// ```no_run
    /// use serde_derive::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// # fn main() {
    /// # let url_to_json_resource = "http://example.org/resource.json";
    /// let user_name = minreq::get(url_to_json_resource)
    ///     .send().unwrap()
    ///     .json::<User>().unwrap() // explicitly declared type `User`
    ///     .name;
    /// println!("User name is '{}'", &user_name);
    /// # }
    /// ```
    #[cfg(feature = "json-using-serde")]
    pub fn json<'a, T>(&'a self) -> Result<T, serde_json::Error>
    where
        T: serde::de::Deserialize<'a>,
    {
        serde_json::from_str(&self.body)
    }
}
