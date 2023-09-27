.section .text.start

.global _start
_start:
    mrs x1, mpidr_el1 // Multiprocessor Affinity Register
#    and x2, x1, #0x100
#    cbnz x2, _loop
    and x1, x1, #0xff // [7:0] Affinity level 0. Lowest level affinity field.
    mrs x2, currentel // get the he current Exception level
    cmp x2, #0x4 // EL1
    beq _el1_entry
    cmp x2, #0x8 // EL2
    beq _el2_entry
    b _loop

_el2_entry:
    ldr x3, =(1 << 31) // The Execution state for EL1 is AArch64.
    msr hcr_el2, x3 // Hypervisor Configuration Register

    ldr x3, =((3 << 28) | (3 << 22) | (1 << 20) | (1 << 11)) // Reserved-1
    msr sctlr_el1, x3 // System Control Register for EL1

    ldr x3, =((7 << 6) | 5) // Big-endian, Asynchronous data abort mask, IRQ mask, 
    msr spsr_el2, x3 // Saved Program Status Register (EL2)

    adr x3, _el1_entry 
    msr elr_el2, x3 // Set the Exception Link Register for EL2 to _el1_entry
    eret // return to _el1_entry

_el1_entry:
    mov x19, x1 // save the Multiprocessor Affinity Register
    mov x1, #1
    msr spsel, x1 // Stack Pointer Select, Use SP_ELx for Exception level ELx.
    adrp x0, BOOT_STACK_TOP
    mov x1, 4096
    mul x1, x1, x19 // core id * 4096
    sub x0, x0, x1 // BOOT_STACK_TOP - core id * 4096
    mov sp, x0 // Set sp register, retain 4096 stack space for each core
    cbnz x19, 1f

    adrp x0, BSS_START
    adrp x1, BSS_END
    sub  x2, x1, x0
    mov  x1, xzr
    bl memset // Clear BSS segment, only core 1 runs it.
        
    adrp x0, KERNEL_PAGE_DIRECTORY
    bl populate_page_table
1:  adrp x0, KERNEL_PAGE_DIRECTORY
    bl mmu_init

    mov x0, x19
    ldr x1, =stack_of_core
    blr x1
    mov sp, x0
    mov x0, x19
    mov fp, sp
    ldr x1, =loader_main
    blr x1

_loop:
    wfe
    b _loop

.data
BOOT_STACK:
.align 12
.space 4096 * 4 // max 4 cores, each core has 4096 stack space.
BOOT_STACK_TOP:

.align 12
.global KERNEL_PAGE_DIRECTORY
KERNEL_PAGE_DIRECTORY:
    .space 4096