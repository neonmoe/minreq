/// This is a simple example to demonstrate the usage of this
/// library. The printed string is quite a bit of JSON, which you
/// might want to handle with the `json-using-serde` feature. For
/// that, check out examples/json.rs!

fn main() -> Result<(), minreq::Error> {
    let response = minreq::get("http://httpbin.org/anything")
        .with_body("Hello, world!")
        .send()?;
    let hello = response.as_str()?;
    println!("{}", hello);
    Ok(())
}
