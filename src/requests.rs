use curl::easy::Easy;
use curl::Error;
use std::str;
use std::io::Read;
use http::URL;

/// Sends a GET request to `url`, returns the response or
/// `curl::Error`.
///
/// Curl's `Error` implements `Display`, so it's easy to print out in
/// case of a fire.
///
/// # Examples
///
/// ```
/// // This application prints out your public IP. (Or an error.)
/// match minreq::get("https://httpbin.org/ip") {
///     Ok(response) => println!("Your public IP: {}", response),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
pub fn get<T: Into<URL>>(url: T) -> Result<String, Error> {
    let mut result = Vec::new();
    match curl_get(&url.into(), &mut result) {
        Ok(_) => match str::from_utf8(&result) {
            Ok(result) => Ok(result.to_string()),
            Err(_) => Ok(String::new()),
        },
        Err(err) => Err(err),
    }
}

/// Sends a POST request to `url` with `body`, returns the response or
/// `curl::Error`.
///
/// # Examples
///
/// ```
/// // This posts "hello" to a server, and prints out the response.
/// // (Or an error.)
/// match minreq::post("https://httpbin.org/post", "hello") {
///     Ok(response) => println!("{}", response),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
pub fn post<T: Into<URL>>(url: T, body: &str) -> Result<String, Error> {
    let mut result = Vec::new();
    match curl_post(&url.into(), &mut result, body.as_bytes()) {
        Ok(_) => match str::from_utf8(&result) {
            Ok(result) => Ok(result.to_string()),
            Err(_) => Ok(String::new()),
        },
        Err(err) => Err(err),
    }
}

fn curl_get(url: &str, result: &mut Vec<u8>) -> Result<(), Error> {
    let mut easy = Easy::new();
    easy.url(url)?;

    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        result.extend_from_slice(data);
        Ok(data.len())
    })?;

    transfer.perform()?;
    Ok(())
}

fn curl_post(url: &str, result: &mut Vec<u8>, mut body: &[u8]) -> Result<(), Error> {
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.post(true)?;
    easy.post_field_size(body.len() as u64)?;

    let mut transfer = easy.transfer();
    transfer.read_function(|buffer| Ok(body.read(buffer).unwrap_or(0)))?;
    transfer.write_function(|data| {
        result.extend_from_slice(data);
        Ok(data.len())
    })?;

    transfer.perform()?;
    Ok(())
}
