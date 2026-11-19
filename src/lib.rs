#![no_std]
#![feature(abi_x86_interrupt)]

use core::{arch::asm, panic};

use crate::{interrupts_handler::read_cr3, memory::memory_init, vga_buffer::clear_screen};

mod interrupts_handler;
mod memory;
mod terminal;
mod vga_buffer;

// This is the main rust function.
// It is called at the end of the assembly code.
#[unsafe(no_mangle)]
pub extern "C" fn rust_init(multiboot_info: u64) {
    // First thing to do!
    clear_screen();

    let res = memory_init(multiboot_info);
    if res == Err(()) {
        println!("Error in mutliboot_init");
        panic!()
    }

    println!("Kernel ready to use!!");

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        crate::vga_buffer::WRITER.force_unlock();
    }
    println!("KERNEL PANIC: {}", info);
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
