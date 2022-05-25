pub trait Address {
    fn pa2kva(&self) -> usize;
    fn kva2pa(&self) -> usize;
}

pub trait ArchTrait {
    fn exception_init();

    // Note: kernel runs at privileged mode
    // need to trigger a half process switching
    // Require: a process has been schedule, its
    // context filled in CONTEXT_FRAME, and its
    // page table installed at low address space.
    fn invalidate_tlb();
    fn wait_for_interrupt();
    fn nop();
    fn fault_address() -> usize;
    fn core_id() -> usize;
    fn curent_privilege() -> usize;
}

pub trait ContextFrameTrait {
    fn new(pc: usize, sp: usize, arg: usize, privileged: bool) -> Self;

    // fn syscall_argument(&self, i: usize) -> usize;
    // fn syscall_number(&self) -> usize;
    // fn set_syscall_result(&mut self, v: &crate::syscall::Result);
    fn exception_pc(&self) -> usize;
    fn set_exception_pc(&mut self, pc: usize);
    fn stack_pointer(&self) -> usize;
    fn set_stack_pointer(&mut self, sp: usize);
    fn set_argument(&mut self, arg: usize);
    fn gpr(&self, index: usize) -> usize;
}
