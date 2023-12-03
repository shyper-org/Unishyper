/// Unlike the exception handler in rustpi,
/// (https://github.com/tonnylyz/rustpie/blob/master/src/arch/aarch64/exception.S)
/// We use thread's own stack to store context.
/// But this may disobey ARM's exception handler rule, who knows?
.macro VECTOR handler
    // Current sp is kernel_sp.
    sub	sp, sp, {size_of_context_frame}
    stp x0, x1,  [sp, #(0 * 16)]
    stp x2, x3,  [sp, #(1 * 16)]
    stp x4, x5,  [sp, #(2 * 16)]
    stp x6, x7,  [sp, #(3 * 16)]
    stp x8, x9,  [sp, #(4 * 16)]
    stp x10,x11, [sp, #(5 * 16)]
    stp x12,x13, [sp, #(6 * 16)]
    stp x14,x15, [sp, #(7 * 16)]
    stp x16,x17, [sp, #(8 * 16)]
    stp x18,x19, [sp, #(9 * 16)]
    stp x20,x21, [sp, #(10 * 16)]
    stp x22,x23, [sp, #(11 * 16)]
    stp x24,x25, [sp, #(12 * 16)]
    stp x26,x27, [sp, #(13 * 16)]
    stp x28,x29, [sp, #(14 * 16)]
    mrs x1, spsr_el1
    stp x30, x1, [sp, #(15 * 16)]
    mrs x0, elr_el1
    mrs x1, sp_el0
    stp x0, x1,  [sp, #(16 * 16)]
    // Mark context as "from irq".
	mrs x0, tpidr_el0
    mov x1, 1
    stp x0, x1,  [sp, #(17 * 16)]
    mov fp, 0
    mov x0, sp
    bl  \handler
    b   pop_context
.endm

.macro VECTOR_DISABLED
1:  wfe
    b      1b
.endm

.text
.global _pop_context_first
_pop_context_first:
    mov sp, x0
.global pop_context
pop_context:
    // ldr x0, [sp, #(31 * 8)] // spsr
    // 0b0100_0100
    // IRQ mask bit [7]   : 0
    // FIQ mask bit [6]   : 1
    // AArch64 state[3:0] : 0b0101 (EL1h)
    mov x0, #0x45
    ldr x1, [sp, #(32 * 8)] // elr
    ldr x2, [sp, #(33 * 8)] // sp
    ldr x3, [sp, #(34 * 8)] // tpidr
    msr spsr_el1, x0
    msr elr_el1, x1
    msr sp_el0, x2
	msr tpidr_el0, x3
    ldp x0, x1,  [sp, #(0 * 16)]
    ldp x2, x3,  [sp, #(1 * 16)]
    ldp x4, x5,  [sp, #(2 * 16)]
    ldp x6, x7,  [sp, #(3 * 16)]
    ldp x8, x9,  [sp, #(4 * 16)]
    ldp x10,x11, [sp, #(5 * 16)]
    ldp x12,x13, [sp, #(6 * 16)]
    ldp x14,x15, [sp, #(7 * 16)]
    ldp x16,x17, [sp, #(8 * 16)]
    ldp x18,x19, [sp, #(9 * 16)]
    ldp x20,x21, [sp, #(10 * 16)]
    ldp x22,x23, [sp, #(11 * 16)]
    ldp x24,x25, [sp, #(12 * 16)]
    ldp x26,x27, [sp, #(13 * 16)]
    ldp x28,x29, [sp, #(14 * 16)]
    ldr x30, [sp, #(15 * 16)]
    add	sp, sp, {size_of_context_frame}
    eret

.section .text.vector
.global vectors
vectors:
.align 11

// Current exception level with SP_EL0.
.org 0x000
    VECTOR current_el_sp0_synchronous
.org 0x080
    VECTOR current_el_sp0_irq
.org 0x100
    VECTOR_DISABLED
.org 0x180
    VECTOR_DISABLED

// Current exception level with SP_ELx, x > 0.
.org 0x200
    VECTOR current_el_spx_synchronous
.org 0x280
    VECTOR current_el_spx_irq
.org 0x300
    VECTOR_DISABLED // FIQ
.org 0x380
    VECTOR current_el_spx_serror

// Lower exception level, aarch64
.org 0x400
    VECTOR lower_aarch64_synchronous
.org 0x480
    VECTOR lower_aarch64_irq
.org 0x500
    VECTOR_DISABLED // FIQ
.org 0x580
    VECTOR lower_aarch64_serror

// Lower exception level, aarch32
.org 0x600
    VECTOR_DISABLED
.org 0x680
    VECTOR_DISABLED
.org 0x700
    VECTOR_DISABLED
.org 0x780
    VECTOR_DISABLED
.org 0x800

.end