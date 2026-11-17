// This file is here mainly to handle interrupt 8: double fault
// We need to give a safe stack to use to the CPU
// We do this using the GDT

use lazy_static::lazy_static;
use core::{arch::asm, cell::UnsafeCell};

const TSS_LENGTH: usize = 104;
const STACK_SIZE: usize = 4096 * 5;
const GDT_LENGTH: usize = 8 * 5;

const CODE_SELECTOR: u16 = 0x08;
const DATA_SELECTOR: u16 = 0x10;
const TSS_SELECTOR: u16 = 0x18;

const PRESENT: u8 = 1 << 7;
const NON_SYSTEM: u8 = 1 << 4;
const EXECUTABLE: u8 = 1 << 3;
const READ_WRITE: u8 = 1 << 1;

const LONG_MODE: u8 = 1 << 1;

const CODE_ACCESS: u8 = PRESENT | NON_SYSTEM | EXECUTABLE | READ_WRITE; // 0x9A
const DATA_ACCESS: u8 = PRESENT | NON_SYSTEM | READ_WRITE;

const KERNEL_CODE: u64 = descriptor(CODE_ACCESS, LONG_MODE);
const KERNEL_DATA: u64 = descriptor(DATA_ACCESS, 0);

// Some tests to make sure
const _: () = assert!(KERNEL_CODE == 0x0020_9A00_0000_0000);
const _: () = assert!(KERNEL_DATA == 0x0000_9200_0000_0000);

const _: () = {
    let (low, high) = tss_encoder(0x1122_3344_5566_7788, 103);
    assert!(low == 0x5500_8966_7788_0067);
    assert!(high == 0x0000_0000_1122_3344);
};

#[repr(C,packed(4))]
pub struct Tss {
    reserved0: u32,
    rsp: [u64; 3], // Useful for later user mode apparently 
    reserved1: u64,
    ist: [u64; 7],
    reserved2: u64,
    reserved3: u16,
    iomap_base: u16,
}

#[repr(C,packed)]
struct Gdtr {
    limit: u16,
    base: u64,
}

impl Gdtr {
    fn new (limit: u16, base: u64) -> Self {
        Gdtr { limit: limit, base: base }
    }
}

#[repr(C, align(16))]
struct Stack(UnsafeCell<[u8; STACK_SIZE]>);
unsafe impl Sync for Stack {}

static DF_STACK: Stack = Stack(UnsafeCell::new([0; STACK_SIZE]));

pub struct Gdt {
    // null, kernel code, kernel data, TSS x 2
    gdt: [u64; 5]
}

impl Gdt {
    fn new () -> Self {
        let (low, high) = tss_encoder(&* TSS as *const Tss as u64, (TSS_LENGTH - 1) as u64);
        let gdt_array = [0, KERNEL_CODE, KERNEL_DATA, low, high];
        Gdt { gdt: gdt_array }
    }
}

lazy_static! {
    pub static ref TSS: Tss = Tss::new();
}

lazy_static! {
    pub static ref GDT: Gdt = Gdt::new();
}


const _: () = assert!(size_of::<Tss>() == TSS_LENGTH);

impl Tss {
    fn new() -> Self {    
        let stack_start = DF_STACK.0.get() as u64;
        let stack_top = stack_start + STACK_SIZE as u64;
        let ist: [u64; 7] = [stack_top, 0,0,0,0,0,0];
        Tss { 
            reserved0: 0, 
            rsp: [0; 3], 
            reserved1: 0, 
            ist: ist,
            reserved2: 0, 
            reserved3: 0, 
            iomap_base: TSS_LENGTH as u16, 
        }
    }
}

const fn descriptor(access: u8, flag: u8) -> u64 {
    ((access as u64) << 40) | (((flag as u64) & 0xF) << 52)
}

const fn tss_encoder (base: u64, limit: u64) -> (u64, u64) {
    let low_limit = limit & 0xffff;
    let base_low = (base & 0xffffff) << 16;
    let flag_access = descriptor(0x89, 0);
    let limit_high: u64 = ((limit >> 16) & 0xf) << 48;
    let base_high: u64 = ((base >> 24) & 0xff) << 56;

    let high_base = base >> 32;

    let first_result = low_limit | base_low | flag_access | limit_high | base_high;
    let second_result = high_base;

    (first_result, second_result)
}

pub fn gdt_init () {
    let gdtr = Gdtr::new((GDT_LENGTH - 1) as u16, &GDT.gdt as *const _ as u64);

    unsafe { 
        asm!("lgdt [{}]", 
            in(reg) &gdtr, 
            options(readonly, nostack, preserves_flags)); 

        asm!(
            "push {sel}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            sel = in(reg) CODE_SELECTOR as u64,
            tmp = out(reg) _,
            options(preserves_flags),
        );

        asm!(
            "mov ds, {data:x}",
            "mov es, {data:x}",
            "mov ss, {data:x}",

            data = in(reg) DATA_SELECTOR,
            options(nostack, preserves_flags),
        );

        asm!("ltr {tss:x}",
            tss = in(reg) TSS_SELECTOR,
            options(nostack, preserves_flags),
        );
    }
}