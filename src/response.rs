use crate::http;
use std::collections::HashMap;
use std::str;

/// An HTTP response.
///
/// The behaviour is as follows:
///
/// 1. When the `Response` is created, the headers of the response are
/// read, and used to initialize this struct.
///
/// 2. Now that you have a `Response`, you can either prepare it for
/// usage by simply calling `load()`, which reads the rest of the
/// message. Alternatively, you can iterate through the returned
/// response while it is being loaded, with `as_iter()`.
///
/// The function [`load_str()`](#method.load_str) exists to enable
/// writing nice, concise one-liners. In case you want to hold on to
/// the Response for a while, and read it later,
/// [`as_str()`](#method.as_str) and [`as_bytes()`](#method.as_bytes)
/// are the functions for you.
pub struct Response {
    /// The status code of the response, eg. 404.
    pub status_code: i32,
    /// The reason phrase of the response, eg. "Not Found".
    pub reason_phrase: String,
    /// The headers of the response.
    pub headers: HashMap<String, String>,
    /// The body of the response.
    body: Vec<u8>,
}

impl Response {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Response {
        let (status_code, reason_phrase) = http::parse_status_line(&bytes);
        let (headers, body_bytes) = http::parse_http_response_content(&bytes);
        Response {
            status_code,
            reason_phrase,
            headers,
            body: body_bytes,
        }
    }

    /// Loads the rest of the HTTP response synchronously.
    ///
    /// If the response has been loaded, this does nothing.
    #[allow(unused_mut)]
    pub fn load(mut self) -> Response {
        // TODO: load the stuff
        self
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
    /// let response = minreq::get(url).send()?.load();
    /// println!("{}", response.as_str()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.body)
    }

    /// Returns a reference to the bytes returned so far.
    ///
    /// Usage:
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let url = "http://example.org/";
    /// let response = minreq::get(url).send()?.load();
    /// println!("{:?}", response.as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.body
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

    /// Loads the rest of the HTTP response synchronously. Ensures
    /// that `self.body` is not `None` after calling this.
    fn load_body_sync(&mut self) {
        // TODO: Implement me!
        unimplemented!();
    }
}
