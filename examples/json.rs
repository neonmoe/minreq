/// This example demonstrates the `json-using-serde` feature.

#[derive(serde::Deserialize)]
struct Response {
    /// The field in which `http://httpbin.org/anything` returns the body.
    data: String,
}

fn main() -> Result<(), minreq::Error> {
    let response = minreq::get("http://httpbin.org/anything")
        .with_body("Hello, world!")
        .send()?;
    let json: Response = response.json()?;
    println!("Hello, world! == {}", &json.data);
    Ok(())
}
