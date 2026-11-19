// This file purpose's is mainly to handle write macros.

use colour::{Colour, VgaAttribute};
use core::fmt;

mod buffer;
mod colour;
mod writer;

pub use writer::WRITER;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}
