use gap_buffer_rust::GapBuffer;

fn main() {
    let mut buf = GapBuffer::new("The_fence!");
    println!("{buf:?}");

    buf.move_left().expect("Error moving left");
    buf.move_left().expect("Error moving left");
    //buf.move_left().expect("Error moving left");
    println!("{buf:?}");

    //buf.move_right().unwrap();
    //buf.move_right().unwrap();
    //buf.move_right().unwrap();
    println!("{buf:?}");

    buf.move_cursor(5).unwrap();
    println!("{buf:?}");

    println!("{}", buf.fetch(0, 9).unwrap());
    println!("{}", buf.fetch(0, 8).unwrap());
    println!("{}", buf.fetch(0, 7).unwrap());
    println!("{}", buf.fetch(0, 6).unwrap());
    println!("{}", buf.fetch(0, 5).unwrap());
    println!("{}", buf.fetch(0, 4).unwrap());
    println!("{}", buf.fetch(0, 3).unwrap());
    println!("{}", buf.fetch(0, 2).unwrap());
    println!("{}", buf.fetch(0, 1).unwrap());
    println!("{}\n", buf.fetch(0, 0).unwrap());
    println!("{}", buf.fetch(1, 9).unwrap());
    println!("{}", buf.fetch(2, 9).unwrap());
    println!("{}", buf.fetch(3, 9).unwrap());
    println!("{}", buf.fetch(4, 9).unwrap());
    println!("{}", buf.fetch(5, 9).unwrap());
    println!("{}", buf.fetch(6, 9).unwrap());
    println!("{}", buf.fetch(7, 9).unwrap());
    println!("{}", buf.fetch(8, 9).unwrap());
    println!("{}", buf.fetch(9, 9).unwrap());

    println!("{buf:?}");

    buf.insert("heyheyhey");
    println!("{buf:?}");
    buf.backspace();
    println!("{buf:?}");
    buf.delete();
    println!("{buf:?}");

    let mut buf2 = GapBuffer::new("Hey Monica, hey!");
    println!("{buf2:?}");
    buf2.move_cursor(11);
    println!("{buf2:?}");
    buf2.insert("xxxxxxxxxxxxxxxx");
    println!("{buf2:?}");
    buf2.insert("x");
    println!("{buf2:?}");
}
