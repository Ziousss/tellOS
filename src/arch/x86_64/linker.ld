ENTRY(start)
SECTIONS {
    . = 1M;
    __kernel_start = .;
    .boot : { KEEP(*(.multiboot_header)) }
    .text : ALIGN(4K) { *(.text .text.*) }
    .rodata : ALIGN(4K) { *(.rodata .rodata.*) *(.data.rel.ro .data.rel.ro.*) }
    .data : ALIGN(4K) { *(.data .data.*) }
    .bss : ALIGN(4K) { *(.bss .bss.*) *(COMMON) }
    . = ALIGN(4K);
    __kernel_end = .;
}