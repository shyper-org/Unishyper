.equ SIZE_OF_CONTEXT_FRAME, 0x118

/// Unlike the exception handler in rustpi,
/// (https://github.com/tonnylyz/rustpie/blob/master/src/arch/aarch64/exception.S)
/// We use thread's own stack to store context.
/// But this may disobey ARM's exception handler rule, who knows?
.macro VECTOR handler
    // Current sp is kernel_sp.
    sub	sp, sp, #SIZE_OF_CONTEXT_FRAME
    // First we save some general registers.
    stp x2, x3,  [sp, #(1 * 16)]
    // Get user sp.
    mrs x3, sp_el0
    sub	x3, x3, #SIZE_OF_CONTEXT_FRAME
    stp x0, x1,  [x3, #(0 * 16)]
    ldp x0, x1,  [sp, #(1 * 16)]
    stp x0, x1,  [x3, #(1 * 16)]
    stp x4, x5,  [x3, #(2 * 16)]
    stp x6, x7,  [x3, #(3 * 16)]
    stp x8, x9,  [x3, #(4 * 16)]
    stp x10,x11, [x3, #(5 * 16)]
    stp x12,x13, [x3, #(6 * 16)]
    stp x14,x15, [x3, #(7 * 16)]
    stp x16,x17, [x3, #(8 * 16)]
    stp x18,x19, [x3, #(9 * 16)]
    stp x20,x21, [x3, #(10 * 16)]
    stp x22,x23, [x3, #(11 * 16)]
    stp x24,x25, [x3, #(12 * 16)]
    stp x26,x27, [x3, #(13 * 16)]
    stp x28,x29, [x3, #(14 * 16)]
    mrs x1, spsr_el1
    stp x30, x1, [x3, #(15 * 16)]
    mrs x0, elr_el1
    mrs x1, sp_el0
    stp x0, x1,  [x3, #(16 * 16)]
    // Mark context as "from irq".
    mov x0, 1
    str x0, [x3, #(34 * 8)]
    mov fp, 0
    mov x0, x3
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
    msr spsel, #0x01
    sub	sp, sp, #SIZE_OF_CONTEXT_FRAME
.global pop_context
pop_context:
    mov x3, x0
    // ldr x0, [x3, #(31 * 8)] // spsr
    // 0b0100_0100
    // IRQ mask bit [7]   : 0
    // FIQ mask bit [6]   : 1
    // AArch64 state[3:0] : 0b0100 (EL1t)
    mov x0, #0x44
    ldr x1, [x3, #(32 * 8)] // elr
    ldr x2, [x3, #(33 * 8)] // sp
    msr spsr_el1, x0
    msr elr_el1, x1
    msr sp_el0, x2
    ldp x0, x1,  [x3, #(1 * 16)]
    stp x0, x1,  [sp, #(1 * 16)] 
    ldp x0, x1,  [x3, #(0 * 16)]
    # ldp x2, x3,  [sp, #(1 * 16)]
    ldp x4, x5,  [x3, #(2 * 16)]
    ldp x6, x7,  [x3, #(3 * 16)]
    ldp x8, x9,  [x3, #(4 * 16)]
    ldp x10,x11, [x3, #(5 * 16)]
    ldp x12,x13, [x3, #(6 * 16)]
    ldp x14,x15, [x3, #(7 * 16)]
    ldp x16,x17, [x3, #(8 * 16)]
    ldp x18,x19, [x3, #(9 * 16)]
    ldp x20,x21, [x3, #(10 * 16)]
    ldp x22,x23, [x3, #(11 * 16)]
    ldp x24,x25, [x3, #(12 * 16)]
    ldp x26,x27, [x3, #(13 * 16)]
    ldp x28,x29, [x3, #(14 * 16)]
    ldr x30, [x3, #(15 * 16)]
    ldp x2, x3,  [sp, #(1 * 16)]
    add	sp, sp, #SIZE_OF_CONTEXT_FRAME
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