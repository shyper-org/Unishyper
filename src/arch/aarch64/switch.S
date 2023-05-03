.equ SIZE_OF_CONTEXT_FRAME, 0x118

.text
.global save_context_on_current_stack
save_context_on_current_stack:
    // Save original sp at x10.
    mov x10, sp

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
    // mrs x1, spsr_el1
    // Save return address and SPSR_EL1.
    str x30, [sp, #(15 * 16)]   // x30
    // Save ELR_EL1(just the return address) and Stack Pointer.
    stp x30, x10,  [sp, #(16 * 16)]  // elr, sp
    // Mark context as "from yield".
    mov x0, 0
    str x0, [sp, #(34 * 8)]

    // Reset frame pointer.
    // mov fp, 0
    
    // Set current SP as first argument and jump tp `switch_to_next_stack`.
    mov x0, sp
    bl switch_to_next_stack
    
    // Get if yield from irq or yield.
    ldr x1, [x0, #(34 * 8)]
    //
    // If x0 is not zero, the next thread is scheduled from timer interrupt.
    // Pop whole context from stack.
    // Note: "first run" thread is yield from irq by default.
    cbnz x1, _pop_context_first

    // If x1 is zero, the next thread is scheduled from thread_yield.
    mov sp, x0

    // Pop x19-x28, because they are expected to be preserved by call routine.
    ldp x18,x19, [sp, #(9 * 16)]
    ldp x20,x21, [sp, #(10 * 16)]
    ldp x22,x23, [sp, #(11 * 16)]
    ldp x24,x25, [sp, #(12 * 16)]
    ldp x26,x27, [sp, #(13 * 16)]
    ldp x28,x29, [sp, #(14 * 16)]
    // Pop original x30 from stack.
    ldr x30, [sp, #(15 * 16)]

    ldp x0, x1,  [sp, #(16 * 16)] // elr->x10, sp->x11

    mov sp, x1
    br x0

