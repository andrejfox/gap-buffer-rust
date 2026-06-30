use gap_buffer_rust::GapBuffer;

fn main() {
    let mut buf = GapBuffer::new("The_fence!");

    buf.move_cursor(5).unwrap();
    println!("{buf:?}");
}
