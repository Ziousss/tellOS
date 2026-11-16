%include "helpers.inc"

global start
extern long_mode_start

section .bss
boot_stack_bottom:
    resb 131072
boot_stack_top:

section .rodata
gdt64:
    dq 0
.code: equ $ - gdt64
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53)
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .text
bits 32

start:
    mov esp, boot_stack_top

    call check_multiboot
    call check_cpuid
    call check_long_mode

    call setup_page_tables
    call enable_paging

    lgdt [gdt64.pointer]
    jmp gdt64.code:long_mode_start

