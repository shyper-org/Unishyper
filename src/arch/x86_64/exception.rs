use alloc::boxed::Box;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{drivers::apic, libs::interrupt::InterruptController};

pub fn init_idt() {
    let idt = Box::leak(Box::new(InterruptDescriptorTable::new()));
    // Set breakpoint handler, caused by `x86_64::instructions::interrupts::int3();`
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    // Set double fault handler.
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.overflow.set_handler_fn(overflow_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);
    idt.stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    // Set page fault handler.
    idt.page_fault.set_handler_fn(page_fault_handler);
    // Set timer handler.
    idt[apic::INT_TIMER].set_handler_fn(timer_interrupt_handler);

    idt.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    println!(
        "EXCEPTION: DOUBLE FAULT error code {}\n{:#?}",
        error_code, stack_frame
    );
    hlt_loop();
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    println!(
        "EXCEPTION: INVALID TSS error code {}\n{:#?}",
        error_code, stack_frame
    );
    hlt_loop();
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!(
        "EXCEPTION: SEGMENT NOT PRESENT error code {}\n{:#?}",
        error_code, stack_frame
    );
    hlt_loop();
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!(
        "EXCEPTION: STACK SEGMENT FAULT error code {}\n{:#?}",
        error_code, stack_frame
    );
    hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!(
        "EXCEPTION: GENERAL PROTECTION FAULT error code {}\n{:#?}",
        error_code, stack_frame
    );
    hlt_loop();
}

use super::hlt_loop;
use x86_64::structures::idt::PageFaultErrorCode;
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    trace!("timer interrupt");
    trace!("stack frame:\n{:#?}", _stack_frame);
    crate::libs::timer::interrupt();
    // Finished interrupt before switching
    apic::INTERRUPT_CONTROLLER.finish(apic::INT_TIMER);
    // Give up CPU actively.
    crate::libs::thread::thread_yield();
}
