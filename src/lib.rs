#![feature(lang_items)]
#![allow(internal_features)]
#![no_std]
#![feature(abi_x86_interrupt)]

use crate::vga_buffer::clear_screen;

mod vga_buffer;
mod interrupts_handler;
mod terminal;

// This is the main rust function.
// It is called at the end of the assembly code.
#[unsafe(no_mangle)]
pub extern "C" fn rust_init() {
    clear_screen();
    interrupts_handler::init();
    println!("Terminal ready to use!!");

    // Put in another file for clarity.
    terminal::run();
}

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}