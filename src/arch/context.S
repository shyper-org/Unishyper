.equ SIZE_OF_CONTEXT_FRAME, 0x110

.text
.global set_cpu_context
.global save_context
save_context:
    // Save original sp at x10.
    mov x10, sp
    // Get kernel stack.
    mov sp, x0
    // Alloc sp space for context frame.
    sub	sp, sp, #SIZE_OF_CONTEXT_FRAME
    stp x18,x19, [sp, #(9 * 16)]
    stp x20,x21, [sp, #(10 * 16)]
    stp x22,x23, [sp, #(11 * 16)]
    stp x24,x25, [sp, #(12 * 16)]
    stp x26,x27, [sp, #(13 * 16)]
    stp x28,x29, [sp, #(14 * 16)]
    str x30, [sp, #(15 * 16)]   // x30, spsr
    stp x30, x10,  [sp, #(16 * 16)]  // elr, sp
    mov fp, 0
    mov x0, sp
    bl set_cpu_context
    // Pop x0 to x3, in case the thread is yielded firstly.
    ldp x0, x1,  [sp, #(0 * 16)]
    ldp x2, x3,  [sp, #(1 * 16)]
    ldp x18,x19, [sp, #(9 * 16)]
    ldp x20,x21, [sp, #(10 * 16)]
    ldp x22,x23, [sp, #(11 * 16)]
    ldp x24,x25, [sp, #(12 * 16)]
    ldp x26,x27, [sp, #(13 * 16)]
    ldp x28,x29, [sp, #(14 * 16)]
    ldr x30, [sp, #(15 * 16)]
    ldp x10, x11,  [sp, #(16 * 16)] // elr, sp
    # add	sp, sp, #SIZE_OF_CONTEXT_FRAME
    mov sp, x11
    br x10

