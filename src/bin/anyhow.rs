fn main() -> Result<(), anyhow::Error> {
    std::fs::remove_file("/this/file/does/not/exist")?;
    Ok(())
}
