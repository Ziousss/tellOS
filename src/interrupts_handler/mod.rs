use core::arch::asm;
use crate::interrupts_handler::{handlers::{KEYBOARD_VECTOR, TIMER_VECTOR, PIC_1_OFFSET, PIC_2_OFFSET, keyboard_handler}, pic::ChainedPics};
use handlers::{breakpoint_handler, timer_handler};

use self::interrupt_table::{
    Idt, IdtEntry, PRESENT, DPL_KERNEL, GATE_INTERRUPT,
};

mod interrupt_table;
mod pic;
mod handlers;

pub use handlers::pop_key;



static mut IDT: Idt = Idt::new();

pub fn read_cs() -> u16 {
    let cs: u16;
    unsafe {
        asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags));
    }
    cs
}

pub fn init() {
    let selector = read_cs();
    let attribute = PRESENT | DPL_KERNEL | GATE_INTERRUPT;
    let entry_3: IdtEntry = IdtEntry::new(breakpoint_handler as *const () as usize as u64, attribute, selector);
    let timer_entry = IdtEntry::new(timer_handler as *const() as usize as u64, attribute, selector);
    let keyboard_entry = IdtEntry::new(keyboard_handler as *const() as usize as u64, attribute, selector);

    unsafe {
        let idt: &'static mut Idt = &mut *(&raw mut IDT);
        idt.set_entry(3, entry_3);
        idt.set_entry(TIMER_VECTOR, timer_entry);
        idt.set_entry(KEYBOARD_VECTOR, keyboard_entry);
        idt.load();
    }

    let mut pics = ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET);
    pics.initialise();

    unsafe { asm!("sti", options(nomem, nostack)); }
}