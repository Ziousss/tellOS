use crate::{memory::{get_kernel_end, get_range, multiboot_mem::{MemoryMap, parse_memory_struct}}, println};

pub fn memory_init(multiboot_info: u64) -> Result<(), ()> {
    let mbi_ptr = multiboot_info as *const u32;
    let total_size = unsafe { mbi_ptr.read_unaligned() };
    
    let res = parse_memory_struct(mbi_ptr, total_size);
    if res.is_none() {
        println!("res is none");
        return Err(());
    }

    let memory_array = res.unwrap();

    let boot_info_end = multiboot_info + total_size as u64;
    let kernel_end = get_kernel_end();

    let usable_range = get_range(&memory_array, kernel_end, boot_info_end);
    if usable_range.is_none(){
        println!("usable_range is none");
        return Err(())
    }

    Ok(())
}