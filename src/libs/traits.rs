// Todo: replace this trait with PAddr and VAddr.
pub trait Address {
    fn pa2kva(&self) -> usize;
    fn kva2pa(&self) -> usize;
}

pub trait ArchTrait {
    fn exception_init();
    fn page_table_init();

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
    /// Get context frame's execption return address.
    fn exception_pc(&self) -> usize;
    /// Set context frame's execption return address.
    /// During thread context initialization process,
    /// exception pc is set as the thread's entry pc address,
    /// and use 'eret' to jump to entry.
    fn set_exception_pc(&mut self, pc: usize);
    /// Get context frame's stack pointer.
    fn stack_pointer(&self) -> usize;
    /// Set context frame's stack pointer.
    fn set_stack_pointer(&mut self, sp: usize);
    /// Get context frame's general purpose register value of given index.
    /// Note: the callee may check the index's legality(x0-x30 on aarch 64).
    fn gpr(&self, index: usize) -> usize;
    /// Set context frame's general purpose register value of given index.
    /// Note: the callee may check the index's legality(x0-x30 on aarch 64).
    fn set_gpr(&mut self, index: usize, value: usize);
    /// Mark the thread is yield from irq.
    /// During context restore processes, from irq will end in 'pop_context_first'.
    fn set_from_irq(&mut self);
    /// Mark the thread is yield yield,
    /// which means, the thread called the `thread_yield` function to give up the cpu cycle.
    /// During context restore processes, from irq will not end in 'pop_context_first'.
    /// See switch.S for details.
    fn set_from_yield(&mut self);
}

pub trait ArchPageTableEntryTrait {
    fn from_pte(value: usize) -> Self;
    fn from_pa(pa: usize) -> Self;
    fn to_pte(&self) -> usize;
    fn to_pa(&self) -> usize;
    fn to_kva(&self) -> usize;
    fn valid(&self) -> bool;
    fn blocked(&self) -> bool;
    fn entry(&self, index: usize) -> Self;
    fn set_entry(&self, index: usize, value: Self);
    fn make_table(frame_pa: usize) -> Self;
  }
