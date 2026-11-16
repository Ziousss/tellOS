// This file contains all the function to handle the
// different interrupt the OS might encounter.

use crate::{interrupts_handler::pic::{CMD_END, MASTER_COMMAND, SLAVE_COMMAND, inb, outb}, println};
use core::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize};
use core::sync::atomic::Ordering;
use super::interrupt_table::InterruptStackFrame;

static TIMER: AtomicU64 = AtomicU64::new(0);

static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);
static BUFFER: [AtomicU8; BUFFER_SIZE] = [const { AtomicU8::new(0) }; BUFFER_SIZE];
static SCANCODE_TO_ASCII: [u8; 128] = build_lower_table();

const BUFFER_SIZE: usize = 128;

pub(super) const PIC_1_OFFSET: u8 = 32;
pub(super) const PIC_2_OFFSET: u8 = 40;

pub(super) const TIMER_VECTOR: u8 = PIC_1_OFFSET;
pub(super) const KEYBOARD_VECTOR: u8 = PIC_1_OFFSET + 1;

pub extern "x86-interrupt" fn breakpoint_handler(_frame: InterruptStackFrame) {
    println!("RIP = {:#x}", _frame.rip);
}

pub extern "x86-interrupt" fn timer_handler(_frame: InterruptStackFrame) {
    TIMER.fetch_add(1, Ordering::Relaxed);
    end_of_interrupt(TIMER_VECTOR);
}

pub extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    let value = unsafe { inb(0x60) };

    if (value & 0x80) == 0 {
        let ascii = SCANCODE_TO_ASCII[value as usize];
        
        if ascii != 0 {
            push_key(ascii);
        }
    }

    end_of_interrupt(KEYBOARD_VECTOR);
}

#[allow(dead_code)]
pub fn read_timer() -> u64 {
    TIMER.load(Ordering::Relaxed)
}

pub fn end_of_interrupt(vector: u8) {
    unsafe {
        if vector >= PIC_2_OFFSET {
            outb(SLAVE_COMMAND, CMD_END);
        }
        outb(MASTER_COMMAND, CMD_END);
    }
}

// Creates a table linking "key press number" to letter.
// This table is lower because it does not handle flags
// such as SHIFT or CTRL.
const fn build_lower_table() -> [u8; 128] {
    let mut t = [0u8; 128];

    // each row starts at a known scancode
    let rows: [(usize, &[u8]); 4] = [
        (0x02, b"1234567890-="),
        (0x10, b"qwertyuiop[]"),
        (0x1E, b"asdfghjkl;'`"),
        (0x2B, b"\\zxcvbnm,./"),
    ];

    let mut r = 0;
    while r < rows.len() {
        let (start, keys) = rows[r];
        let mut i = 0;
        while i < keys.len() {
            t[start + i] = keys[i];
            i += 1;
        }
        r += 1;
    }

    t[0x39] = b' ';    // space
    t[0x1C] = b'\n';   // enter
    t[0x0E] = 8;       // backspace, handled separately later
    t
}

fn push_key(ascii: u8) {
    let head = HEAD.load(Ordering::Relaxed);
    let next = (head + 1) % BUFFER_SIZE;
    let tail = TAIL.load(Ordering::Acquire);

    if next == tail {
        return
    }

    BUFFER[head].store(ascii, Ordering::Relaxed);
    HEAD.store(next, Ordering::Release);
}

pub fn pop_key() -> Option<u8> {
    let tail = TAIL.load(Ordering::Relaxed);
    let head = HEAD.load(Ordering::Acquire);

    if head == tail {
        return None;
    }

    let ascii = BUFFER[tail].load(Ordering::Relaxed);
    TAIL.store((tail + 1) % BUFFER_SIZE, Ordering::Release);
    
    Some(ascii)
}