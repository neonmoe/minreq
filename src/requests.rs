use http::{Method, Request, Response, URL};
use connection::Connection;

// TODO: Fix the example
/// Sends a GET request to `url`, returns the response or
/// `curl::Error`.
///
/// Curl's `Error` implements `Display`, so it's easy to print out in
/// case of a fire.
///
/// # Examples
///
/// ```ignore
/// // This application prints out your public IP. (Or an error.)
/// match minreq::get("https://httpbin.org/ip") {
///     Ok(response) => println!("Your public IP: {}", response),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
pub fn get<T: Into<URL>>(url: T) -> Response {
    let request = Request::new(Method::Get, url.into());
    let connection = Connection::new(request);
    connection.send()
}

// TODO: Fix the example
/// Sends a POST request to `url` with `body`, returns the response or
/// `curl::Error`.
///
/// # Examples
///
/// ```ignore
/// // This posts "hello" to a server, and prints out the response.
/// // (Or an error.)
/// match minreq::post("https://httpbin.org/post", "hello") {
///     Ok(response) => println!("{}", response),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
pub fn post<T: Into<URL>>(url: T, body: String) -> Response {
    let request = Request::new(Method::Post, url.into()).with_body(body);
    let connection = Connection::new(request);
    connection.send()
}
