fn main() -> Result<(), std::io::Error> {
    std::fs::remove_file("/this/file/does/not/exist")
}
