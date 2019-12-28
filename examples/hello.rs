/// This is a simple example to demonstrate the usage of this
/// library. The printed string is quite a bit of JSON, which you
/// might want to handle with the `json-using-serde` feature. For
/// that, check out examples/json.rs!

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), minreq::Error> {
    let response = minreq::get("http://httpbin.org/anything")
        .with_body("Hello, world!")
        .send()?;
    let hello = response.as_str()?;
    println!("{}", hello);
    Ok(())
}

/// See this example for wasm target:
/// ```
/// use wasm_bindgen::prelude::*;
/// #[wasm_bindgen]
/// pub async fn test() {
///     let response = minreq::get("http://httpbin.org/anything")
///         .with_body("Hello, world!")
///         .send().await.unwrap();
///     let hello = response.as_str().unwrap();
///     println!("{}", hello);
/// }
/// ```
/// 
/// The example feature of cargo is not able to run this please ignore the following code.
#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("can't use the cargo example feature for wasm target")
}