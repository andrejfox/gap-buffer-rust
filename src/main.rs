use std::fmt;
use std::{mem::MaybeUninit, slice};

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

const MIN_SIZE: usize = 16;
impl GapBuffer {
    fn new<T: AsRef<str>>(string: T) -> Self {
        let s = string.as_ref();
        let size;
        if s.len() < MIN_SIZE {
            size = MIN_SIZE;
        } else {
            size = s.len().next_power_of_two();
        }
        let mut data: Box<[MaybeUninit<char>]> = Box::new_uninit_slice(size * 2);

        for (i, c) in s.chars().enumerate() {
            data[i].write(c);
        }

        Self {
            size: size * 2,
            text_size: s.len(),
            gap_start: s.len(),
            gap_end: size * 2 - 1,
            data: data,
        }
    }

    pub fn insert<T: AsRef<str>>(&mut self, string: T) {
        let s = string.as_ref();

        while s.len() + self.text_size > self.size {
            self.double_buffer();
        }

        for (i, c) in s.chars().enumerate() {
            self.data[i + self.gap_start].write(c);
        }

        self.gap_start += s.len();
        self.text_size += s.len();
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
        if self.gap_end >= self.size {
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
                self.move_left().unwrap();
            }
        } else if loc > self.gap_start {
            let delta = loc - self.gap_start;

            for _ in 0..delta {
                self.move_right().unwrap();
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

    fn fetch(&self, x: usize, y: usize) -> Result<String, GapBufferError> {
        if x > y || y >= self.text_size {
            return Err(GapBufferError::InvalidRange);
        }

        let mut out = String::new();

        let left_end = y.min(self.gap_start - 1);
        if x < self.gap_start {
            let left: &[char] = unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(x) as *const char, left_end - x + 1)
            };

            out.extend(left.iter().copied());
        }

        if y >= self.gap_start {
            let right_start;
            let right_end;

            if x >= self.gap_start {
                right_start = x - (self.gap_start - 1) + self.gap_end;
                right_end = y - x + 1;
            } else {
                right_start = self.gap_end + 1;
                right_end = y - (self.gap_start - 1);
            }

            let right: &[char] = unsafe {
                slice::from_raw_parts(
                    self.data.as_ptr().add(right_start) as *const char,
                    right_end,
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

        // can change to meet personal prefrence
        let row = MIN_SIZE;
        let end = self.size / row;
        for i in 0..end {
            write!(f, "| ")?;
            for j in (i * row)..((i + 1) * row) {
                //println!("j:{j} s:{} e:{}", self.gap_start, self.gap_end());
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
        write!(f, "")
    }
}
