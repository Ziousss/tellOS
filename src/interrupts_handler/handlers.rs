// This file contains all the function to handle the
// different interrupt the OS might encounter.

use crate::{interrupts_handler::pic::{CMD_END, MASTER_COMMAND, SLAVE_COMMAND, inb, outb}, println};
use core::sync::atomic::{Ordering, AtomicBool, AtomicU8, AtomicU64, AtomicUsize};
use super::interrupt_table::InterruptStackFrame;

static TIMER: AtomicU64 = AtomicU64::new(0);

static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);
static BUFFER: [AtomicU8; BUFFER_SIZE] = [const { AtomicU8::new(0) }; BUFFER_SIZE];
static SCANCODE_TO_LOWER_ASCII: [u8; 128] = build_lower_table();
static SCANCODE_TO_UPPER_ASCII: [u8; 128] = build_caps_table();


const BUFFER_SIZE: usize = 128;
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
static CAPS_LOCK_PRESSED: AtomicBool = AtomicBool::new(false);
static ALT_PRESSED: AtomicBool = AtomicBool::new(false);

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

    match value {
        0x1d => CTRL_PRESSED.store(true, Ordering::Relaxed),
        0x9d => CTRL_PRESSED.store(false, Ordering::Relaxed),
        0x2a | 0x36 => SHIFT_PRESSED.store(true, Ordering::Relaxed),
        0xaa | 0xb6 => SHIFT_PRESSED.store(false, Ordering::Relaxed),
        0x3a => { CAPS_LOCK_PRESSED.fetch_xor(true, Ordering::Relaxed); }
        0x38 => ALT_PRESSED.store(true, Ordering::Relaxed),
        0xb8 => ALT_PRESSED.store(false, Ordering::Relaxed),
        v if (v & 0x80) == 0 => {
            if let Some(c) = translate(value) {
                push_key(c);
            }
        }
        _ => {}
    }

    end_of_interrupt(KEYBOARD_VECTOR);
}

fn translate(value: u8) -> Option<u8> {
    let lower = SCANCODE_TO_LOWER_ASCII[value as usize];
    if lower == 0 {
        return None;
    }

    let shift = SHIFT_PRESSED.load(Ordering::Relaxed);
    let caps = CAPS_LOCK_PRESSED.load(Ordering::Relaxed);
    let ctrl = CTRL_PRESSED.load(Ordering::Relaxed);
    let alt = ALT_PRESSED.load(Ordering::Relaxed);

    if ctrl {
        return if lower.is_ascii_lowercase() { Some(lower & 0x1F) } else { None };
    }

    if alt {
        push_alt(lower);
        return None;
    }

    if lower.is_ascii_lowercase() {
        if shift ^ caps {
            Some(lower.to_ascii_uppercase())
        } else {
            Some(lower)
        }
    } else if shift {
        Some(SCANCODE_TO_UPPER_ASCII[value as usize])
    } else {
        Some(lower)
    }
}

fn push_alt(lower: u8) {
    let head = HEAD.load(Ordering::Relaxed);
    let tail = TAIL.load(Ordering::Relaxed);
    let used = (head + BUFFER_SIZE - tail) % BUFFER_SIZE;

    if used + 2 < BUFFER_SIZE {
        push_key(0x1b);
        push_key(lower);
    }
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

const fn build_caps_table() -> [u8; 128] {
    let mut t = [0u8; 128];

    // each row starts at a known scancode
    let rows: [(usize, &[u8]); 4] = [
        (0x02, b"!@#$%^&*()_+"),
        (0x10, b"QWERTYUIOP{}"),
        (0x1E, b"ASDFGHJKL:\"~"),
        (0x2B, b"|ZXCVBNM<>?"),
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