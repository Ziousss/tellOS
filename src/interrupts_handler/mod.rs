use core::arch::asm;
use crate::interrupts_handler::{gdt::gdt_init, handlers::{KEYBOARD_VECTOR, PIC_1_OFFSET, PIC_2_OFFSET, TIMER_VECTOR, divide_error_handler, double_fault_handler, gpf_handler, invalid_opcode_handler, keyboard_handler, page_fault_handler}, pic::{ChainedPics, pit_init}};
use handlers::{breakpoint_handler, timer_handler};

use self::interrupt_table::{
    Idt, IdtEntry, PRESENT, DPL_KERNEL, GATE_INTERRUPT,
};

mod interrupt_table;
mod pic;
mod handlers;
mod gdt;

pub use handlers::{pop_key, read_timer_seconds};

static mut IDT: Idt = Idt::new();

const DOUBLE_FAULT_IST: u8 = 1;

pub fn read_cs() -> u16 {
    let cs: u16;
    unsafe {
        asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags));
    }
    cs
}

pub fn read_cr2() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

// Initialises the Interrupt Descriptor Table (IDT) and configures
// the interrupt handlers.
//
// Then initialises a chained Programmable Interrupt Controllers
// (PICs), remapping hardware IRQs to the configured interrupt offsets.
//
// Finally, we execute sti to enable hardware interrupts.
pub fn init() {
    gdt_init();

    let selector = read_cs();
    let attribute = PRESENT | DPL_KERNEL | GATE_INTERRUPT;

    let entry_0 = IdtEntry::new(divide_error_handler as *const () as usize as u64, attribute, selector);
    let entry_3 = IdtEntry::new(breakpoint_handler as *const () as usize as u64, attribute, selector);
    let entry_6 = IdtEntry::new(invalid_opcode_handler as *const () as usize as u64, attribute, selector);
    let entry_8 = IdtEntry::new(double_fault_handler as *const () as usize as u64, attribute, selector).with_ist(DOUBLE_FAULT_IST);
    let entry_13 = IdtEntry::new(gpf_handler as *const () as usize as u64, attribute, selector);
    let entry_14 = IdtEntry::new(page_fault_handler as *const () as usize as u64, attribute, selector);
    let timer_entry = IdtEntry::new(timer_handler as *const() as usize as u64, attribute, selector);
    let keyboard_entry = IdtEntry::new(keyboard_handler as *const() as usize as u64, attribute, selector);

    unsafe {
        let idt: &'static mut Idt = &mut *(&raw mut IDT);
        idt.set_entry(0, entry_0);
        idt.set_entry(3, entry_3);
        idt.set_entry(6, entry_6);
        idt.set_entry(8, entry_8);
        idt.set_entry(13, entry_13);
        idt.set_entry(14, entry_14);

        idt.set_entry(TIMER_VECTOR, timer_entry);
        idt.set_entry(KEYBOARD_VECTOR, keyboard_entry);
        idt.load();
    }

    let mut pics = ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET);
    pics.initialise();

    pit_init();

    unsafe { asm!("sti", options(nomem, nostack)); }
}