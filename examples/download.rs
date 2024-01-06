/// This example shows how a [`Response`] is a [`std::io::Read`],
/// so it can be easily copied to [`std::io::write`], such as stdout
/// or a file.
fn main() -> Result<(), minreq::Error> {
    let mut response = minreq::get("http://example.com").send_lazy()?;
    std::io::copy(&mut response, &mut std::io::stdout().lock())?;
    Ok(())
}
