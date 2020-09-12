fn main() -> Result<(), anyhow::Error> {
    use anyhow::Context;
    std::fs::remove_file("/this/file/does/not/exist").context("oh no")?;
    Ok(())
}
