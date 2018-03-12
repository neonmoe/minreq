// A small program to post a thing to a server.
extern crate minreq;

fn main() {
    // Send a request
    match minreq::post("https://requestb.in/yourkey", "hello") {
        // Post was completed successfully, ignore
        Ok(_) => (),
        // Request failed for some reason, print out the error
        Err(err) => println!("[ERROR]: {}", err),
    }
}
