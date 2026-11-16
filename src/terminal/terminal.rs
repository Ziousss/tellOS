// This file is here to handle everything that has a link
// with the OS' terminal.

use crate::print;
use core::arch::asm;
use crate::interrupts_handler;

const LINE_CAPACITY: usize = 256;

struct Line {
    buf: [u8; LINE_CAPACITY],
    len: usize,
    alt_pending: bool,
}

impl Line {
    fn new() -> Self {
        Line {
            buf: [0u8; LINE_CAPACITY],
            len: 0,
            alt_pending: false
        }
    }
}

pub fn run() {
    let mut line_struct = Line::new();
    prompt();

    //Waiting for something to happen.
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }

        while let Some(c) = interrupts_handler::pop_key() {            
            handle_key(c, &mut line_struct);
        }
    }
}

fn prompt() {
    print!("tellOS-Terminal: ");
}

// Here to handle special input such that 0x08 (del), etc.
fn handle_key(c: u8, line_struct: &mut Line) {
    if c == 0x1B {
        line_struct.alt_pending = true;
        return;
    }

    if line_struct.alt_pending {
        line_struct.alt_pending = false;
        handle_alt(c, line_struct);
        return;
    }

    match c {
        0x08 => {
            if line_struct.len > 0 {
                delete_last(line_struct);
            }
        }
        b'\n' => {
            print!("\n");
            line_struct.len = 0;
            prompt();
        }
        0x03 => {
            print!("^C\n");
            line_struct.len = 0;
            prompt();
        }
        0x15 => {
            while line_struct.len > 0 {
                delete_last(line_struct);
            }
        }
        c if c < 0x20 => {}

        _ => {
            if line_struct.len < LINE_CAPACITY {
                line_struct.buf[line_struct.len] = c;
                line_struct.len += 1;
                print!("{}", c as char);
            }
        }
    }
}

fn handle_alt(value: u8, line_struct: &mut Line) {
    match value {
        0x08 => {
            while line_struct.len > 0 && line_struct.buf[line_struct.len - 1] == b' ' {      // skip trailing spaces
                delete_last(line_struct);
            }
            while line_struct.len > 0 && line_struct.buf[line_struct.len - 1] != b' ' {      // delete the word
                delete_last(line_struct);
            }
        }

        _ => {}
    }
}

fn delete_last(line_struct: &mut Line) {
    line_struct.len -= 1;
    line_struct.buf[line_struct.len] = 0;
    print!("\x08");
}