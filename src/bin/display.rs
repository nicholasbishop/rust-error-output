fn main() {
    let err = std::fs::remove_file("/this/file/does/not/exist").unwrap_err();
    eprintln!("{}", err);
}
