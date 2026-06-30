use std::mem::MaybeUninit;
use std::{fmt, slice};

const MIN_SIZE: usize = 16;

#[derive(Clone)]
pub struct GapBuffer {
    size: usize,
    text_size: usize,
    gap_start: usize,
    gap_end: usize,
    data: Box<[MaybeUninit<char>]>,
}

#[derive(Debug)]
pub enum GapBufferError {
    OutOfBounds,
    CursorAtEnd,
    InvalidRange,
}
impl GapBuffer {
    pub fn new<T: AsRef<str>>(string: T) -> Self {
        let s = string.as_ref();
        let num_of_chars = s.chars().count();

        let size;
        if num_of_chars < MIN_SIZE {
            size = MIN_SIZE;
        } else {
            size = num_of_chars.next_power_of_two();
        }

        let mut data: Box<[MaybeUninit<char>]> = Box::new_uninit_slice(size * 2);

        for (i, c) in s.chars().enumerate() {
            data[i].write(c);
        }

        Self {
            size: size * 2,
            text_size: num_of_chars,
            gap_start: num_of_chars,
            gap_end: size * 2 - 1,
            data,
        }
    }
}

impl GapBuffer {
    pub fn insert<T: AsRef<str>>(&mut self, string: T) {
        let s = string.as_ref();
        let num_of_chars = s.chars().count();

        while num_of_chars + self.text_size > self.size {
            self.double_buffer();
        }

        for (i, c) in s.chars().enumerate() {
            self.data[i + self.gap_start].write(c);
        }

        self.gap_start += num_of_chars;
        self.text_size += num_of_chars;
    }

    fn double_buffer(&mut self) {
        let mut new_data: Box<[MaybeUninit<char>]> = Box::new_uninit_slice(self.size * 2);

        new_data[..self.gap_start].copy_from_slice(&self.data[..self.gap_start]);

        new_data[(self.size + self.gap_end + 1)..]
            .copy_from_slice(&self.data[(self.gap_end + 1)..]);

        self.data = new_data;
        self.gap_end += self.size;
        self.size *= 2;
    }

    pub fn delete(&mut self) -> Result<(), GapBufferError> {
        if self.gap_end >= self.size - 1 {
            return Err(GapBufferError::CursorAtEnd);
        }

        self.text_size -= 1;
        self.gap_end += 1;

        Ok(())
    }

    pub fn backspace(&mut self) -> Result<(), GapBufferError> {
        if self.gap_start <= 0 {
            return Err(GapBufferError::CursorAtEnd);
        }

        self.text_size -= 1;
        self.gap_start -= 1;

        Ok(())
    }

    pub fn move_cursor(&mut self, loc: usize) -> Result<(), GapBufferError> {
        if loc > self.text_size {
            return Err(GapBufferError::OutOfBounds);
        }

        if loc < self.gap_start {
            let delta = self.gap_start - loc;

            for _ in 0..delta {
                self.move_left()?;
            }
        } else if loc > self.gap_start {
            let delta = loc - self.gap_start;

            for _ in 0..delta {
                self.move_right()?;
            }
        }

        Ok(())
    }

    pub fn move_left(&mut self) -> Result<(), GapBufferError> {
        if self.gap_start <= 0 {
            return Err(GapBufferError::CursorAtEnd);
        }

        let c = unsafe { self.data[self.gap_start - 1].assume_init_read() };
        self.data[self.gap_end].write(c);

        self.gap_start -= 1;
        self.gap_end -= 1;

        Ok(())
    }

    pub fn move_right(&mut self) -> Result<(), GapBufferError> {
        if self.gap_end >= self.size - 1 {
            return Err(GapBufferError::CursorAtEnd);
        }

        let c = unsafe { self.data[self.gap_end + 1].assume_init_read() };
        self.data[self.gap_start].write(c);

        self.gap_start += 1;
        self.gap_end += 1;

        Ok(())
    }

    pub fn fetch(&self, x: usize, y: usize) -> Result<String, GapBufferError> {
        if x > y || y >= self.text_size {
            return Err(GapBufferError::InvalidRange);
        }

        let mut out = String::new();

        if x < self.gap_start {
            let left_len = y.min(self.gap_start - 1) - x + 1;

            let left: &[char] = unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(x) as *const char, left_len)
            };

            out.extend(left.iter().copied());
        }

        if y >= self.gap_start {
            let right_start;
            let right_len;

            if x >= self.gap_start {
                right_start = self.gap_end + x - self.gap_start + 1;
                right_len = y - x + 1;
            } else {
                right_start = self.gap_end + 1;
                right_len = y - (self.gap_start - 1);
            }

            let right: &[char] = unsafe {
                slice::from_raw_parts(
                    self.data.as_ptr().add(right_start) as *const char,
                    right_len,
                )
            };

            out.extend(right.iter().copied());
        }

        Ok(out)
    }
}

impl fmt::Debug for GapBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "size: {}\ngap_start: {}\ngap_end: {}\n",
            self.size, self.gap_start, self.gap_end
        )?;

        let row = MIN_SIZE;
        let end = self.size / row;

        for i in 0..end {
            write!(f, "| ")?;

            for j in (i * row)..((i + 1) * row) {
                if j == self.gap_start && !(self.gap_start > self.gap_end) {
                    write!(f, "> ")?;
                } else if j == self.gap_end && !(self.gap_start > self.gap_end) {
                    write!(f, "< ")?;
                } else if j < self.gap_start || j > self.gap_end {
                    let c = unsafe { self.data[j].assume_init_ref() };
                    write!(f, "{c} ")?;
                } else {
                    write!(f, "ⓧ ")?;
                }
            }

            if i == end {
                write!(f, "|")?;
                break;
            }

            write!(f, "|\n")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(buf: &GapBuffer) -> String {
        if buf.text_size == 0 {
            String::new()
        } else {
            buf.fetch(0, buf.text_size - 1).unwrap()
        }
    }

    #[test]
    fn correct_fetching() {
        let mut buf = GapBuffer::new("The_fence!");
        buf.move_cursor(5).unwrap();
        let mut s= buf.fetch(0, 9).unwrap();
        assert!(s.contains("The_fence!"));
        s = buf.fetch(0, 8).unwrap();
        assert!(s.contains("The_fence"));
        s = buf.fetch(0, 7).unwrap();
        assert!(s.contains("The_fenc"));
        s = buf.fetch(0, 6).unwrap();
        assert!(s.contains("The_fen"));
        s = buf.fetch(0, 5).unwrap();
        assert!(s.contains("The_fe"));
        s = buf.fetch(0, 4).unwrap();
        assert!(s.contains("The_f"));
        s = buf.fetch(0, 3).unwrap();
        assert!(s.contains("The_"));
        s = buf.fetch(0, 2).unwrap();
        assert!(s.contains("The"));
        s = buf.fetch(0, 1).unwrap();
        assert!(s.contains("Th"));
        s = buf.fetch(0, 0).unwrap();
        assert!(s.contains("T"));

        s = buf.fetch(9, 9).unwrap();
        assert!(s.contains("!"));
        s = buf.fetch(8, 9).unwrap();
        assert!(s.contains("e!"));
        s = buf.fetch(7, 9).unwrap();
        assert!(s.contains("ce!"));
        s = buf.fetch(6, 9).unwrap();
        assert!(s.contains("nce!"));
        s = buf.fetch(5, 9).unwrap();
        assert!(s.contains("ence!"));
        s = buf.fetch(4, 9).unwrap();
        assert!(s.contains("fence!"));
        s = buf.fetch(3, 9).unwrap();
        assert!(s.contains("_fence!"));
        s = buf.fetch(2, 9).unwrap();
        assert!(s.contains("e_fence!"));
        s = buf.fetch(1, 9).unwrap();
        assert!(s.contains("he_fence!"));
        s = buf.fetch(0, 9).unwrap();
        assert!(s.contains("The_fence!"));
    }

    #[test]
    fn buffer_expands_correctly() {
        let mut buf = GapBuffer::new("Hey Monica, hey!");
        buf.move_cursor(11);
        buf.insert("xxxxxxxxxxxxxxxx");
        assert_eq!(buf.size, 32);
        buf.insert("x");
        assert_eq!(buf.size, 64);
    }

    #[test]
    fn new_empty() {
        let buf = GapBuffer::new("");

        assert_eq!(text(&buf), "");
        assert_eq!(buf.text_size, 0);
    }

    #[test]
    fn new_with_text() {
        let buf = GapBuffer::new("hello");

        assert_eq!(text(&buf), "hello");
        assert_eq!(buf.text_size, 5);
    }

    #[test]
    fn insert_at_end() {
        let mut buf = GapBuffer::new("hello");

        buf.insert(" world");

        assert_eq!(text(&buf), "hello world");
    }

    #[test]
    fn insert_at_beginning() {
        let mut buf = GapBuffer::new("world");

        buf.move_cursor(0).unwrap();
        buf.insert("hello ");

        assert_eq!(text(&buf), "hello world");
    }

    #[test]
    fn insert_in_middle() {
        let mut buf = GapBuffer::new("helo");

        buf.move_cursor(2).unwrap();
        buf.insert("l");

        assert_eq!(text(&buf), "hello");
    }

    #[test]
    fn multiple_insertions() {
        let mut buf = GapBuffer::new("");

        buf.insert("abc");
        buf.insert("def");
        buf.insert("ghi");

        assert_eq!(text(&buf), "abcdefghi");
    }

    #[test]
    fn move_cursor_left() {
        let mut buf = GapBuffer::new("abcdef");

        buf.move_cursor(3).unwrap();
        buf.insert("X");

        assert_eq!(text(&buf), "abcXdef");
    }

    #[test]
    fn move_cursor_right() {
        let mut buf = GapBuffer::new("abcdef");

        buf.move_cursor(2).unwrap();
        buf.move_cursor(6).unwrap();
        buf.insert("!");

        assert_eq!(text(&buf), "abcdef!");
    }

    #[test]
    fn move_cursor_to_start() {
        let mut buf = GapBuffer::new("abc");

        buf.move_cursor(0).unwrap();
        buf.insert("X");

        assert_eq!(text(&buf), "Xabc");
    }

    #[test]
    fn move_cursor_to_end() {
        let mut buf = GapBuffer::new("abc");

        buf.move_cursor(3).unwrap();
        buf.insert("X");

        assert_eq!(text(&buf), "abcX");
    }

    #[test]
    fn move_cursor_out_of_bounds() {
        let mut buf = GapBuffer::new("abc");

        assert!(matches!(
            buf.move_cursor(4),
            Err(GapBufferError::OutOfBounds)
        ));
    }

    #[test]
    fn move_left_at_start_fails() {
        let mut buf = GapBuffer::new("abc");

        buf.move_cursor(0).unwrap();

        assert!(matches!(buf.move_left(), Err(GapBufferError::CursorAtEnd)));
    }

    #[test]
    fn move_right_at_end_fails() {
        let mut buf = GapBuffer::new("abc");

        buf.move_cursor(3).unwrap();

        assert!(matches!(buf.move_right(), Err(GapBufferError::CursorAtEnd)));
    }

    #[test]
    fn backspace_middle() {
        let mut buf = GapBuffer::new("abcd");

        buf.move_cursor(2).unwrap();
        buf.backspace().unwrap();

        assert_eq!(text(&buf), "acd");
    }

    #[test]
    fn backspace_beginning_fails() {
        let mut buf = GapBuffer::new("abcd");

        buf.move_cursor(0).unwrap();

        assert!(matches!(buf.backspace(), Err(GapBufferError::CursorAtEnd)));
    }

    #[test]
    fn delete_middle() {
        let mut buf = GapBuffer::new("abcd");

        buf.move_cursor(2).unwrap();
        buf.delete().unwrap();

        assert_eq!(text(&buf), "abd");
    }

    #[test]
    fn delete_end_fails() {
        let mut buf = GapBuffer::new("abcd");

        buf.move_cursor(4).unwrap();

        assert!(matches!(buf.delete(), Err(GapBufferError::CursorAtEnd)));
    }

    #[test]
    fn fetch_whole_string() {
        let buf = GapBuffer::new("hello world");

        assert_eq!(buf.fetch(0, 10).unwrap(), "hello world");
    }

    #[test]
    fn fetch_left_side() {
        let mut buf = GapBuffer::new("abcdef");

        buf.move_cursor(3).unwrap();

        assert_eq!(buf.fetch(0, 2).unwrap(), "abc");
    }

    #[test]
    fn fetch_right_side() {
        let mut buf = GapBuffer::new("abcdef");

        buf.move_cursor(2).unwrap();

        assert_eq!(buf.fetch(3, 5).unwrap(), "def");
    }

    #[test]
    fn fetch_across_gap() {
        let mut buf = GapBuffer::new("abcdef");

        buf.move_cursor(3).unwrap();

        assert_eq!(buf.fetch(1, 4).unwrap(), "bcde");
    }

    #[test]
    fn fetch_single_character() {
        let buf = GapBuffer::new("abcdef");

        assert_eq!(buf.fetch(2, 2).unwrap(), "c");
    }

    #[test]
    fn fetch_invalid_reversed_range() {
        let buf = GapBuffer::new("abcdef");

        assert!(matches!(buf.fetch(4, 2), Err(GapBufferError::InvalidRange)));
    }

    #[test]
    fn fetch_out_of_bounds() {
        let buf = GapBuffer::new("abcdef");

        assert!(matches!(buf.fetch(0, 6), Err(GapBufferError::InvalidRange)));
    }

    #[test]
    fn insert_causes_buffer_growth() {
        let mut buf = GapBuffer::new("");

        let s = "x".repeat(100);

        buf.insert(&s);

        assert_eq!(text(&buf), s);
    }

    #[test]
    fn insert_after_growth_in_middle() {
        let mut buf = GapBuffer::new("hello");

        buf.move_cursor(2).unwrap();

        let s = "x".repeat(100);
        buf.insert(&s);

        assert_eq!(text(&buf), format!("he{}llo", s));
    }

    #[test]
    fn complex_edit_sequence() {
        let mut buf = GapBuffer::new("Hello");

        buf.move_cursor(5).unwrap();
        buf.insert(" World");

        buf.move_cursor(6).unwrap();
        buf.insert("Beautiful ");

        buf.move_cursor(5).unwrap();
        buf.delete().unwrap();

        buf.move_cursor(0).unwrap();
        buf.insert("> ");

        assert_eq!(text(&buf), "> HelloBeautiful World");
    }

    #[test]
    fn repeated_cursor_moves_do_not_change_text() {
        let mut buf = GapBuffer::new("abcdefghijklmnopqrstuvwxyz");

        for i in 0..=26 {
            buf.move_cursor(i).unwrap();
        }

        for i in (0..=26).rev() {
            buf.move_cursor(i).unwrap();
        }

        assert_eq!(text(&buf), "abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn repeated_insert_delete() {
        let mut buf = GapBuffer::new("");

        for _ in 0..50 {
            buf.insert("a");
        }

        assert_eq!(text(&buf), "a".repeat(50));

        buf.move_cursor(0).unwrap();

        for _ in 0..50 {
            buf.delete().unwrap();
        }

        assert_eq!(text(&buf), "");
    }
}
