use multiboot_mem::MemoryMap;
use crate::println;

mod addr_trans;
mod init;
mod multiboot_mem;

pub use init::memory_init;

const MAPPED_LIMIT: u64 = 0x800000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: u64,
    pub end: u64,
}

unsafe extern "C" {
    static __kernel_end: u8;
}

pub fn get_kernel_end() -> u64 {
    return &raw const __kernel_end as u64
}

pub fn get_range(memory_array: &MemoryMap, kernel_end: u64, boot_info_end: u64) -> Option<Range>{    
    let highest_used = kernel_end.max(boot_info_end).max(0x10_0000);

    let base = (highest_used + 4095) & !4095;
    let mut region_end_opt = None;

    for i in 0..memory_array.len {
        let e = &memory_array.entry_array[i];
        let end_e = e.base_addr.saturating_add(e.length);
        if e.region_type == 1 && e.base_addr <= base && base < end_e {
            region_end_opt = Some(end_e);
            break;
        }
    }

    if region_end_opt.is_none(){
        return None
    }
    let region_end = region_end_opt.unwrap();
    let limit = region_end.min(MAPPED_LIMIT) & !4095;

    if limit <= base {
        println!("Base limit condition not met");
        return None
    }
    println!("region_end = {:#x}, limit = {:#x}, base = {:#x}", region_end, limit, base);
    println!("Kernel end = {:#x}, boot_info_end = {:#x}", kernel_end, boot_info_end);


    return Some (Range {
        start: base,
        end: limit,
    })
}