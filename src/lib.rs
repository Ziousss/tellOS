#![feature(lang_items)]
#![allow(internal_features)]
#![no_std]

use crate::vga_buffer::clear_screen;
mod vga_buffer;

#[unsafe(no_mangle)]
pub extern "C" fn rust_init() {
    clear_screen();
    print!("Hello world!");
}

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}