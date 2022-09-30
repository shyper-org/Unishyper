.equ SIZE_OF_CONTEXT_FRAME, 0x110

.text
.global set_cpu_context
.global save_context
save_context:
    msr daifset, 0b111
    // Choose to use SP_EL1 as current stack pointer.
    mov x1, #1
    msr spsel, x1
    // Get kernel stack.
    mov sp, x0
    // Alloc sp space for context frame.
    sub	sp, sp, #SIZE_OF_CONTEXT_FRAME
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
    // Save the spsr_el1 here, may it's not necessary.
    mrs x1, spsr_el1
    stp x30, x1, [sp, #(15 * 16)]   // x30, spsr
    mrs x1, sp_el0
    stp x30, x1,  [sp, #(16 * 16)]  // elr, sp
    mov fp, 0
    mov x0, sp
    bl set_cpu_context
    // Pop the whole context in case the thread is yielded firstly.
    ldr x0, [sp, #(31 * 8)] // spsr
    ldr x1, [sp, #(32 * 8)] // elr
    ldr x2, [sp, #(33 * 8)] // sp
    msr spsr_el1, x0
    msr elr_el1, x1
    msr sp_el0, x2
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
    // Set x30 as the return PC.
    ldr x30, [sp, #(15 * 16)]
    // If the thread is yielded firstly, register after x3 is useless.
    // If the thread is yielded before, we don't need to care about x10.
    # ldr x10, [sp, #(33 * 8)]
    add	sp, sp, #SIZE_OF_CONTEXT_FRAME
    # mov sp, x10
    msr daifclr, 0b111
    // Return to PC at x30.
    eret

