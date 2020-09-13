---
title: testx
---

## Explicit panic

```rust
fn main() {
    panic!("oh no");
}
```
```
thread 'main' panicked at 'oh no', src/bin/panic.rs:2:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Unwrap

```rust
fn main() {
    std::fs::remove_file("/this/file/does/not/exist").unwrap();
}
```
```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }', src/bin/unwrap.rs:2:55
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Expect

```rust
fn main() {
    std::fs::remove_file("/this/file/does/not/exist").expect("oh no");
}
```
```
thread 'main' panicked at 'oh no: Os { code: 2, kind: NotFound, message: "No such file or directory" }', src/bin/expect.rs:2:55
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Return `Result` from `main`

```rust
fn main() -> Result<(), std::io::Error> {
    std::fs::remove_file("/this/file/does/not/exist")
}
```
```
Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

## Display

```rust
fn main() {
    let err = std::fs::remove_file("/this/file/does/not/exist").unwrap_err();
    eprintln!("{}", err);
}
```
```
No such file or directory (os error 2)
```

## Debug

```rust
fn main() {
    let err = std::fs::remove_file("/this/file/does/not/exist").unwrap_err();
    eprintln!("{:?}", err);
}
```
```
Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

## Anyhow

```rust
fn main() -> Result<(), anyhow::Error> {
    std::fs::remove_file("/this/file/does/not/exist")?;
    Ok(())
}
```
```
Error: No such file or directory (os error 2)
```

## Anyhow with context

```rust
fn main() -> Result<(), anyhow::Error> {
    use anyhow::Context;
    std::fs::remove_file("/this/file/does/not/exist").context("oh no")?;
    Ok(())
}
```
```
Error: oh no

Caused by:
    No such file or directory (os error 2)
```