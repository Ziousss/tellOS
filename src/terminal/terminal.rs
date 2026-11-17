// This file is here to handle everything that has a link
// with the OS' terminal.

use crate::terminal::built_in::{help_builtin, uptime_builtin};
use crate::{print, println};
use core::{arch::asm, str::from_utf8};
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
            command_handle(line_struct);
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

fn command_handle(line_struct: &mut Line) {
    let len = line_struct.len;
    let text = from_utf8(&line_struct.buf[..len]).unwrap_or("");
    let mut words = text.split_ascii_whitespace();

    let command = words.next();
    let arg_1 = words.next();

    println!();

    match (command, arg_1) {
        (Some("uptime"), Some(_)) => print!("Uptime takes no argument"),
        (Some("uptime"), None) => uptime_builtin(),
        (Some("help"), None) => help_builtin(),
        (Some("help"), Some(_)) => print!("Help takes no argument."),
        (Some(other), _) => print!("{}: command not found", other),
        (None, _) => {}
    }
}