# gap-buffer-rust

A simple gap buffer implementation in Rust.

## Features

- Insert
- Delete
- Backspace
- Cursor movement
- Efficient editing near the cursor
- Minimal dependencies

## Example

```rust
use gap_buffer::GapBuffer;

let mut buf = GapBuffer::new("hello");
buf.move_cursor(5).unwrap();
buf.insert(" world");

assert_eq!(buf.fetch(0, 10).unwrap(), "hello world");