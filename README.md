# gap-buffer-rust

[![Build & Test](https://github.com/andrejfox/gap-buffer-rust/actions/workflows/build_and_test.yml/badge.svg)](https://github.com/andrejfox/gap-buffer-rust/actions/workflows/build_and_test.yml)

A simple gap buffer implementation in Rust.

```toml
# You can just add it to Cargo.toml as such:
[dependencies]
gap_buffer = { git = "https://github.com/andrejfox/gap-buffer-rust.git" }
```

## Features

- Insert
- Delete
- Backspace
- Cursor movement
- Efficient editing near the cursor

## Example

```rust
use gap_buffer::GapBuffer;

let mut buf = GapBuffer::new("hello");
buf.move_cursor(5).unwrap();
buf.insert(" world");

assert_eq!(buf.fetch(0, 10).unwrap(), "hello world");
