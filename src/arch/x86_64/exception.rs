use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::set_general_handler;

use crate::drivers::apic;
use crate::drivers::InterruptController;
use crate::libs::traits::InterruptControllerTrait;

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn load_idt() {
    unsafe {
        IDT.load_unsafe();
    }
}

pub fn init_idt() {
    let idt = unsafe { &mut *(&mut IDT as *mut _ as *mut InterruptDescriptorTable) };

    set_general_handler!(idt, abort, 0..32);
    set_general_handler!(idt, unhandle, 32..64);
    set_general_handler!(idt, unknown, 64..);

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
    idt[apic::ERROR_INTERRUPT_NUMBER as usize].set_handler_fn(error_interrupt_handler);
    idt[apic::SPURIOUS_INTERRUPT_NUMBER as usize].set_handler_fn(spurious_interrupt_handler);

    // idt.load();
}

fn abort(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
    error!("Exception {index}");
    error!("Error code: {error_code:?}");
    error!("Stack frame: {stack_frame:#?}");
    crate::libs::thread::thread_exit()
    // scheduler::abort();
}

fn unhandle(_stack_frame: InterruptStackFrame, index: u8, _error_code: Option<u64>) {
    warn!("received unhandled irq {index}");
    InterruptController::finish(index as usize);
}

fn unknown(_stack_frame: InterruptStackFrame, index: u8, _error_code: Option<u64>) {
    warn!("unknown interrupt {index}");
    InterruptController::finish(index as usize);

    // apic::eoi();
}

pub fn irq_install_handler(irq_number: u32, handler: usize) {
    info!(
        "Install handler for interrupt {} handler {:#x}",
        irq_number, handler
    );

    let idt = unsafe { &mut *(&mut IDT as *mut _ as *mut InterruptDescriptorTable) };
    unsafe {
        idt[apic::IRQ_MIN + irq_number as usize].set_handler_addr(x86_64::VirtAddr::new(
            u64::try_from(handler as usize).unwrap(),
        ));
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    hlt_loop();
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
    #[cfg(feature = "zone")]
    let cur_pkru = zone::rdpkru();
    #[cfg(feature = "zone")]
    let _ = zone::switch_to_privilege();

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    #[cfg(feature = "zone")]
    if error_code.contains(PageFaultErrorCode::PROTECTION_KEY) {
        println!(
            "\nMEMORY PROTECTION KEY VIOLATION on {}!!! current PKRU {:#x}\n",
            crate::libs::thread::current_thread_id(),
            cur_pkru,
        );

        crate::arch::page_table::page_table()
            .lock()
            .dump_entry_flags_of_va(Cr2::read().as_u64() as usize);
    }
    println!("{:#?}", stack_frame);

    crate::libs::thread::thread_exit();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    #[cfg(feature = "zone")]
    let ori_pkru = zone::switch_to_privilege();

    // debug!("timer_interrupt_handler");
    // trace!("stack frame:\n{:#?}", _stack_frame);
    crate::libs::timer::interrupt();
    // Finished interrupt before switching
    InterruptController::finish(apic::INT_TIMER);

    #[cfg(feature = "zone")]
    zone::switch_from_privilege(ori_pkru);

    // Give up CPU actively.
    crate::libs::thread::thread_yield();
}

extern "x86-interrupt" fn error_interrupt_handler(stack_frame: InterruptStackFrame) {
    error!("APIC LVT Error Interrupt");
    error!("ESR: {:#?}", unsafe { apic::local_apic().error_flags() });
    error!("{:#?}", stack_frame);
    InterruptController::finish(apic::ERROR_INTERRUPT_NUMBER as usize);
    crate::libs::thread::thread_exit();
}

extern "x86-interrupt" fn spurious_interrupt_handler(stack_frame: InterruptStackFrame) {
    error!("Spurious Interrupt: {:#?}", stack_frame);
    crate::libs::thread::thread_exit();
}
