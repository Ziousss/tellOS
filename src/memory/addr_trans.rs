use crate::read_cr3;

const ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

pub fn addr_translation(virt: u64) -> Option<u64> {
    let p4_index = (virt >> 39) & 0x1ff;
    let p3_index = (virt >> 30) & 0x1ff;
    let p2_index = (virt >> 21) & 0x1ff;
    let p1_index = (virt >> 12) & 0x1ff;
    let offset = virt & 0xfff;

    let cr3 = read_cr3();
    let mut p1_entry;

    unsafe {
        let p4_entry = read_entry(cr3, p4_index)?;
        let p3_entry = read_entry(p4_entry, p3_index)?;
        let p2_entry = read_entry(p3_entry, p2_index)?;
        p1_entry = read_entry(p2_entry, p1_index)?;
    }

    let phys_addr = (p1_entry & ADDR_MASK) | offset;

    Some(phys_addr)
}

unsafe fn read_entry(table: u64, index: u64) -> Option<u64> {
    let pointer = (table & ADDR_MASK) as *const u64;
    let entry = unsafe { pointer.add(index as usize).read_volatile() };

    if entry & 1 == 0 {
        return None;
    }

    Some(entry)
}
