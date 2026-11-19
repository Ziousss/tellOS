use crate::println;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MemoryMapEntry {
    base_addr: u64,
    length: u64,
    region_type: u32,
    reserved: u32,
}

const MAX_REGIONS: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct MemoryMap {
    entry_array: [MemoryMapEntry; MAX_REGIONS],
    len: usize,
}

impl MemoryMapEntry {
    fn new() -> Self {
        MemoryMapEntry {
            base_addr: 0,
            length: 0,
            region_type: 0,
            reserved: 0,
        }
    }
}

impl MemoryMap {
    fn new() -> Self {
        MemoryMap {
            entry_array: [MemoryMapEntry::new(); MAX_REGIONS],
            len: 0,
        }
    }

    fn push(&mut self, entry: MemoryMapEntry) -> bool {
        if self.len == MAX_REGIONS {
            return false;
        }

        self.entry_array[self.len] = entry;
        self.len += 1;
        true
    }
}

const MULTIBOOT_TAG_TYPE_MMAP: u32 = 6;

pub fn memory_init(multiboot_info: u64) -> Result<(), ()> {
    let res = parse_memory_struct(multiboot_info);
    if res.is_none() {
        return Err(());
    }

    let memory_array = res.unwrap();

    Ok(())
}

fn parse_memory_struct(multiboot_info: u64) -> Option<MemoryMap> {
    let mbi_ptr = multiboot_info as *const u32;
    let total_size = unsafe { mbi_ptr.read_unaligned() };

    let tag =
        unsafe { find_multiboot_tags(multiboot_info, total_size, MULTIBOOT_TAG_TYPE_MMAP as u32)? };

    let mem_size = unsafe { tag.add(4).cast::<u32>().read_unaligned() } as usize;
    let entry_size = unsafe { tag.wrapping_add(8).cast::<u32>().read_unaligned() } as usize;
    if entry_size < size_of::<MemoryMapEntry>() {
        return None;
    }

    let mut offset = 16;
    let mut memory_array = MemoryMap::new();

    while offset + entry_size <= mem_size {
        let entry_ptr = unsafe { tag.add(offset) };
        let entry = unsafe { entry_ptr.cast::<MemoryMapEntry>().read_unaligned() };

        if !memory_array.push(entry) {
            println!("Len too high for MemoryMap");
            return None;
        }
        offset += entry_size;
    }

    for i in 0..memory_array.len {
        println!(
            "memory slot {} = {:#x}, {:#x}, {:#x}, {:#x}",
            i + 1,
            memory_array.entry_array[i].base_addr,
            memory_array.entry_array[i].length,
            memory_array.entry_array[i].region_type,
            memory_array.entry_array[i].reserved
        );
    }

    Some(memory_array)
}

unsafe fn find_multiboot_tags(
    mbi_addr: u64,
    total_size: u32,
    tag_to_find: u32,
) -> Option<*const u8> {
    let mbi = mbi_addr as *const u8;
    let mut offset: usize = 8;

    while offset + 8 <= total_size as usize {
        let tag = mbi.wrapping_add(offset).cast::<u32>();

        let tag_type = unsafe { tag.read_unaligned() };

        let tag_size = unsafe { tag.add(1).read_unaligned() };

        if tag_size < 8 || offset + tag_size as usize > total_size as usize {
            println!("Invalid Multiboot2 tag!");
            return None;
        }

        if tag_type == 0 {
            break;
        }

        if tag_type == tag_to_find {
            return Some(mbi.wrapping_add(offset));
        }

        offset = (offset + tag_size as usize + 7) & !7;
    }

    None
}
