use crate::print;
use core::arch::asm;
use crate::interrupts_handler;

pub fn run() {
    let mut line_len: usize = 0;
    prompt();

    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }

        while let Some(c) = interrupts_handler::pop_key() {            
            handle_key(c, &mut line_len);
        }
    }
}

fn prompt() {
    print!("tellOS-Terminal: ");
}

fn handle_key(c: u8, line_len: &mut usize) {
    match c {
        0x08 => {
            if *line_len > 0 {
                print!("{}", '\x08');
                *line_len -= 1;
            }
        }
        b'\n' => {
            print!("{}", '\n');
            *line_len = 0;
            prompt();
        }
        _ => {
            print!("{}", c as char);
            *line_len += 1;
        }
    }
}