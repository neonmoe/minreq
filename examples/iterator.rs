/// This example demonstrates probably the most complicated part of
/// `minreq`. Useful when making loading bars, for example.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), minreq::Error> {
    let mut buffer = Vec::new();
    for byte in minreq::get("http://httpbin.org/get").send_lazy()? {
        // The connection could have a problem at any point during the
        // download, so each byte needs to be unwrapped.
        let (byte, len) = byte?;

        // The `byte` is the current u8 of data we're iterating
        // through.
        print!("{}", byte as char);

        // The `len` is the expected amount of incoming bytes
        // including the current one: this will be the rest of the
        // body if the server provided a Content-Length header, or
        // just the size of the remaining chunk in chunked transfers.
        buffer.reserve(len);
        buffer.push(byte);

        // Flush the printed text so each char appears on your
        // terminal right away.
        flush();

        // Wait for 50ms so the data doesn't appear instantly fast
        // internet connections, to demonstrate that the body is being
        // printed char-by-char.
        sleep();
    }
    Ok(())
}

// Helper functions
#[cfg(not(target_arch = "wasm32"))]
fn flush() {
    use std::io::{stdout, Write};
    stdout().lock().flush().ok();
}

#[cfg(not(target_arch = "wasm32"))]
fn sleep() {
    use std::thread::sleep;
    use std::time::Duration;

    sleep(Duration::from_millis(5));
}


/// See this example for wasm target:
/// ```
/// use wasm_bindgen::prelude::*;
/// #[wasm_bindgen]
/// pub async fn test() {
///     unimplemented!();
/// }
/// ```
/// 
/// The example feature of cargo is not able to run this please ignore the following code.
#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("can't use the cargo example feature for wasm target")
}