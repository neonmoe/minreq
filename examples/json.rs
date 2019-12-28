/// This example demonstrates the `json-using-serde` feature.

#[cfg(not(target_arch = "wasm32"))]
#[derive(serde_derive::Deserialize)]
struct Response {
    data: String,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), minreq::Error> {
    let response = minreq::get("http://httpbin.org/anything")
        .with_body("Hello, world!")
        .send()?;
    let json: Response = response.json()?;
    println!("{}", json.data);
    Ok(())
}

/// See this example for wasm target:
/// ```
/// #[derive(serde_derive::Deserialize)]
/// struct Response {
///     data: String,
/// }
///
/// use wasm_bindgen::prelude::*;
/// #[wasm_bindgen]
/// pub async fn test() {
///     let response = minreq::get("http://httpbin.org/anything")
///         .with_body("Hello, world!")
///         .send().await.unwrap();
///     let json: Response = response.json()?;
///     println!("{}", json.data);
/// }
/// ```
/// 
/// The example feature of cargo is not able to run this please ignore the following code.
#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("can't use the cargo example feature for wasm target")
}