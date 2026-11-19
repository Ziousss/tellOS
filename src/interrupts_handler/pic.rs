// This file handles the "creation" and initialisation of
// Programmable Interrupt Controllers (PIC) and chained PIC
// to allow the use of slave command and data.
//
// PIC are crucial as they are the way to handle hardware interrupt..
use core::arch::asm;

pub(super) const MASTER_COMMAND: u16 = 0x20;
pub(super) const SLAVE_COMMAND: u16 = 0xa0;
pub(super) const MASTER_DATA: u16 = 0x21;
pub(super) const SLAVE_DATA: u16 = 0xa1;

pub(super) const CMD_INIT: u8 = 0x11;
pub(super) const CMD_END: u8 = 0x20;
pub(super) const MODE_8086: u8 = 0x01;

pub(super) const WAIT_PORT: u16 = 0x80;

struct Pic {
    offset: u8,
    command: u16,
    data: u16,
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset1,
                    command: MASTER_COMMAND,
                    data: MASTER_DATA,
                },
                Pic {
                    offset: offset2,
                    command: SLAVE_COMMAND,
                    data: SLAVE_DATA,
                },
            ],
        }
    }

    pub fn initialise(&mut self) {
        unsafe {
            outb(self.pics[0].command, CMD_INIT);
            io_wait();
            outb(self.pics[1].command, CMD_INIT);
            io_wait();

            outb(self.pics[0].data, self.pics[0].offset);
            io_wait();
            outb(self.pics[1].data, self.pics[1].offset);
            io_wait();

            outb(self.pics[0].data, 0x4);
            io_wait();
            outb(self.pics[1].data, 0x2);
            io_wait();

            outb(self.pics[0].data, MODE_8086);
            io_wait();
            outb(self.pics[1].data, MODE_8086);
            io_wait();

            outb(self.pics[0].data, 0xfc);
            io_wait();
            outb(self.pics[1].data, 0xff);
            io_wait();
        }
    }
}

pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    unsafe {
        asm!("in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

pub fn io_wait() {
    unsafe {
        outb(WAIT_PORT, 0);
    }
}

const PIT_INPUT_HZ: u32 = 1_193_182;
pub const TIMER_HZ: u32 = 100;
const PIT_DIVISOR: u16 = (PIT_INPUT_HZ / TIMER_HZ) as u16;

const PIT_COMMAND: u8 = 0x36;
const PIT_PORT_COMMAND: u16 = 0x43;
const PIT_CHANNEL_PORT: u16 = 0x40;

pub fn pit_init() {
    unsafe {
        outb(PIT_PORT_COMMAND, PIT_COMMAND);
        outb(PIT_CHANNEL_PORT, (PIT_DIVISOR & 0xff) as u8);
        outb(PIT_CHANNEL_PORT, (PIT_DIVISOR >> 8) as u8);
    }
}
