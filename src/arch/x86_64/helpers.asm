%include "constants.inc"

global check_multiboot
global check_cpuid
global check_long_mode
global setup_page_tables
global enable_paging

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
p1_table:
    resb 4096 * P1_COUNT

section .text
bits 32


error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte [0xb800a], al
    hlt    

check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret 
.no_multiboot:
    mov al, "0"
    jmp error

check_cpuid:
    ; Step 1, store all the flags in eax
    pushfd
    pop eax

    ; Store te initial flags for restoration later
    mov ecx, eax

    ; Flip the 'CPUID' bit
    xor eax, 1 << 21
    
    ; Step 2, copy the changed flags into FLAGS
    push eax
    popfd

    ; Validate the 'CPUID' bit flip worked
    cmp eax, ecx
    je .no_cpuid
    ret

.no_cpuid:
    mov al, "1"
    jmp error

check_long_mode:
    ; eax is the argument and return to CPUID
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29 ; check for the long mode bit
    jz .no_long_mode
    ret

.no_long_mode:
    mov al, "2"
    jmp error

setup_page_tables:
    mov eax, p3_table
    or eax, 0b11
    mov [p4_table], eax

    mov eax, p2_table
    or eax, 0b11
    mov [p3_table], eax

    mov ecx, 0

.map_p2:
    mov eax, ecx
    shl eax, 12
    add eax, p1_table
    or eax, 0b11
    mov [p2_table + ecx * 8], eax
    inc ecx
    cmp ecx, P1_COUNT
    jne .map_p2

    mov ecx, 1

.map_p1:
    mov eax, ecx
    shl eax, 12
    or eax, 0b11
    mov [p1_table + ecx * 8], eax
    inc ecx
    cmp ecx, P1_COUNT * 512
    jne .map_p1

    ret

enable_paging:
    mov eax, p4_table
    mov cr3, eax

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret