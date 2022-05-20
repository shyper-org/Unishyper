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
  }
  