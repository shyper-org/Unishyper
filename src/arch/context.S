.equ SIZE_OF_CONTEXT_FRAME, 0x110

.text
.global set_cpu_context
.global save_context
save_context:
    // Save original sp at x10.
    mov x10, sp
    // Use SP_EL1
    msr spsel, #0x01
    // Get kernel stack.
    mov sp, x0
    // Alloc sp space for context frame.
    sub	sp, sp, #SIZE_OF_CONTEXT_FRAME
    // The called routine is expected to preserve x19-x28.
    stp x18,x19, [sp, #(9 * 16)]
    stp x20,x21, [sp, #(10 * 16)]
    stp x22,x23, [sp, #(11 * 16)]
    stp x24,x25, [sp, #(12 * 16)]
    stp x26,x27, [sp, #(13 * 16)]
    stp x28,x29, [sp, #(14 * 16)]
    // Save the spsr_el1 here, it's necessary!!!
    mrs x1, spsr_el1
    // Save return address and SPSR_EL1.
    stp x30, x1, [sp, #(15 * 16)]   // x30, spsr
    // Save ELR_EL1(just the return address) and Stack Pointer.
    stp x30, x10,  [sp, #(16 * 16)]  // elr, sp
    // Reset stack pointer.
    mov fp, 0
    // Set current SP as first argument and jump tp set_cpu_context.
    mov x0, sp
    bl set_cpu_context
    // Pop whole context from stack.
    // In case the next thread is scheduled from timer interrupt.
    bl pop_context
    # ldr x0, [sp, #(31 * 8)] // spsr
    # ldr x1, [sp, #(32 * 8)] // elr
    # ldr x2, [sp, #(33 * 8)] // sp
    # msr spsr_el1, x0
    # msr elr_el1, x1
    # msr sp_el0, x2
    # ldp x0, x1,  [sp, #(0 * 16)]
    # ldp x2, x3,  [sp, #(1 * 16)]
    # ldp x4, x5,  [sp, #(2 * 16)]
    # ldp x6, x7,  [sp, #(3 * 16)]
    # ldp x8, x9,  [sp, #(4 * 16)]
    # ldp x10,x11, [sp, #(5 * 16)]
    # ldp x12,x13, [sp, #(6 * 16)]
    # ldp x14,x15, [sp, #(7 * 16)]
    # ldp x16,x17, [sp, #(8 * 16)]
    # ldp x18,x19, [sp, #(9 * 16)]
    # ldp x20,x21, [sp, #(10 * 16)]
    # ldp x22,x23, [sp, #(11 * 16)]
    # ldp x24,x25, [sp, #(12 * 16)]
    # ldp x26,x27, [sp, #(13 * 16)]
    # ldp x28,x29, [sp, #(14 * 16)]
    # ldr x30, [sp, #(15 * 16)]
    # eret

