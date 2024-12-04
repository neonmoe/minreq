//! This example demonstrates the `json-using-serde` feature.

fn main() -> Result<(), minreq::Error> {
    let response = minreq::get("http://httpbin.org/anything")
        .with_body("Hello, world!")
        .send()?;

    // httpbin.org/anything returns the body in the json field "data":
    let json: serde_json::Value = response.json()?;
    println!("\"Hello, world!\" == {}", json["data"]);

    Ok(())
}
