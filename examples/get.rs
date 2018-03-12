// A small program to print out the user's ip.
extern crate minreq;

fn main() {
    // Send a request
    match minreq::get("https://api.ipify.org") {
        // Request was completed successfully, print out the result
        Ok(result) => println!("{}", result),
        // Request failed for some reason, print out the error
        Err(err) => println!("[ERROR]: {}", err),
    }
}
