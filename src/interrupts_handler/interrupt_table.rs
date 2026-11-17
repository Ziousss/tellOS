// This file handles eveything regarding the
// Interrupt Decriptor Table (IDT) initialisation.

use core::arch::asm;

const IDT_ENTRY_LENGTH: usize = 16;
const IDT_LENGTH: usize = 4096;
const IDTR_LENGTH: usize = 10;

pub(super) const PRESENT: u8 = 1 << 7;
pub(super) const DPL_KERNEL: u8 = 0 << 5;
pub(super) const GATE_INTERRUPT: u8 = 0xE;

#[allow(dead_code)]
pub(super) const GATE_TRAP: u8 = 0xF;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Idtr {
    limit: u16,
    base: u64,
}

// Need three fields for the address for legacy reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct IdtEntry {
    handler_lowest_16: u16,
    code_segment_selector: u16,
    ist_index: u8,
    type_and_attribute: u8,
    handler_16_31: u16,
    handler_32_63: u32,
    reserved: u32,
}

#[repr(C)]
pub struct Idt {
    entries: [IdtEntry; 256],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64
}

// Making sure the size are right.
const _: () = assert!(size_of::<Idtr>() == IDTR_LENGTH);
const _: () = assert!(size_of::<IdtEntry>() == IDT_ENTRY_LENGTH);
const _: () = assert!(size_of::<Idt>() == IDT_LENGTH);
const _: () = assert!((PRESENT | DPL_KERNEL | GATE_INTERRUPT) == 0x8E);

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            handler_lowest_16: 0,
            code_segment_selector: 0,
            ist_index: 0,
            type_and_attribute: 0,
            handler_16_31: 0,
            handler_32_63: 0,
            reserved: 0,
        }
    }

    pub const fn with_ist(mut self, index: u8) -> Self{
        self.ist_index = index & 0b111;
        self
    } 

    pub fn new(handler_addr: u64, attribute: u8, selector: u16) -> Self {        
        Self { 
            handler_lowest_16: handler_addr as u16, 
            code_segment_selector: selector, 
            ist_index: 0, 
            type_and_attribute: attribute, 
            handler_16_31: (handler_addr >> 16) as u16, 
            handler_32_63: (handler_addr >> 32) as u32, 
            reserved: 0 
        }
    }
}

impl Idt {
    pub const fn new() -> Self {
        Self { entries: [IdtEntry::missing(); 256] }
    }

    pub fn set_entry(&mut self, vector_index: u8, idt_entry: IdtEntry) {
        self.entries[vector_index as usize] = idt_entry;
    }

    pub fn load(&'static self){
        let idtr = Idtr {
            limit: (IDT_LENGTH - 1) as u16,
            base: self as *const Idt as u64,
        };

        unsafe { asm!("lidt [{}]", 
            in(reg) &idtr, 
            options(readonly, nostack, preserves_flags));
        }
    }
}