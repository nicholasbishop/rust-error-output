fn main() {
    std::fs::remove_file("/this/file/does/not/exist").expect("oh no");
}