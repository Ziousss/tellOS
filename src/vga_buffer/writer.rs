use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

use super::buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, Buffer, ScreenChar};
use super::{Colour, VgaAttribute};

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row: 0,
        colour_code: VgaAttribute::from_colour(Colour::White, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        do_line_wrapping: true,
    });
}

pub struct Writer {
    pub(super) column_position: usize,
    pub(super) row: usize,
    pub(super) colour_code: VgaAttribute,
    pub(super) buffer: &'static mut Buffer,
    pub(super) do_line_wrapping: bool,
}

impl Writer {
    // Main Writer function.
    // Takes a byte and see how it should print it depending
    // on its value (and flags such sa SHIFT or CTRL later).
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
            }

            b'\t' => {
                let delta = 4 - (self.column_position % 4);
                for _ in 0..delta {
                    self.write_byte(b' ');
                }
            }

            b'\r' => {
                self.column_position = 0;
            }

            0x08 => {
                if self.column_position == 0 {
                    if self.row == 0 {
                        return;
                    }
                    self.row -= 1;
                    self.column_position = BUFFER_WIDTH;
                }
                self.column_position -= 1;

                let (row, col, colour_code) = (self.row, self.column_position, self.colour_code);
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: b' ',
                    colour_code,
                });
            }

            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    if self.do_line_wrapping {
                        self.new_line()
                    } else {
                        return;
                    }
                }

                let row = self.row;
                let col = self.column_position;
                let colour_code = self.colour_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    colour_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        if self.row == BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    self.buffer.chars[row - 1][col].write(self.buffer.chars[row][col].read())
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row += 1;
        }

        self.column_position = 0;
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }

        self.column_position = 0;
        self.row = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\r' | b'\t' | 0x08 => self.write_byte(byte),
                // non writtable byte.
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
