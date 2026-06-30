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
            data,
        }
    }
}

impl GapBuffer {
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
                right_start = self.gap_end + x - (self.gap_start - 1);
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
