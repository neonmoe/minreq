extern crate curl;

use curl::easy::Easy;
use curl::Error;
use std::str;
use std::io::Read;

/// Sends a get request to `url`, returns the response or
/// `curl::Error`. Curl's `Error` implements `Display`, so it's easy
/// to print out in case of a fire.
///
/// # Examples
///
/// ```
/// match minreq::get("https://api.ipify.org") {
///     Ok(result) => println!("{}", result),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
pub fn get<T: Into<String>>(url: T) -> Result<String, Error> {
    let url = url.into();
    let mut result = Vec::new();
    match curl_get(&url, &mut result) {
        Ok(_) => match str::from_utf8(&result) {
            Ok(result) => Ok(result.to_string()),
            Err(_) => Ok(String::new()),
        },
        Err(err) => Err(err),
    }
}

/// Sends a post request to `url`, returns the error if there is one.
///
/// # Examples
///
/// ```
/// match minreq::post("https://requestb.in/yourkey", "hello") {
///     Ok(_) => (),
///     Err(err) => println!("[ERROR]: {}", err),
/// }
/// ```
pub fn post<T: Into<String>>(url: T, body: T) -> Result<(), Error> {
    let url = url.into();
    let body = body.into();
    curl_post(&url, body.as_bytes())
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

fn curl_post(url: &str, mut body: &[u8]) -> Result<(), Error> {
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.post(true)?;
    easy.post_field_size(body.len() as u64)?;

    let mut transfer = easy.transfer();
    transfer.read_function(|buffer| Ok(body.read(buffer).unwrap_or(0)))?;

    transfer.perform()?;
    Ok(())
}
