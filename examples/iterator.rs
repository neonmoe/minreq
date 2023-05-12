/// This example demonstrates probably the most complicated part of
/// `minreq`. Useful when making loading bars, for example.

fn main() -> Result<(), minreq::Error> {
    let mut buffer = Vec::new();
    for byte in minreq::get("http://example.com").send_lazy()? {
        // The connection could have a problem at any point during the
        // download, so each byte needs to be unwrapped. An IO error
        // of the WouldBlock kind may also be returned, but it is not
        // a fatal error, it just means that we're still waiting for
        // more bytes. Some operating systems just block while waiting
        // for more bytes, others return a WouldBlock error.
        let (byte, len) = match byte {
            Ok((byte, len)) => (byte, len),
            Err(minreq::Error::IoError(err)) if err.kind() == std::io::ErrorKind::WouldBlock => {
                continue
            }
            Err(err) => return Err(err),
        };

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

fn flush() {
    use std::io::{stdout, Write};
    stdout().lock().flush().ok();
}

fn sleep() {
    use std::thread::sleep;
    use std::time::Duration;

    sleep(Duration::from_millis(2));
}
