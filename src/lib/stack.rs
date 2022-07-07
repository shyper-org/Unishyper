use crate::ArchTrait;
use crate::arch::BOARD_CORE_NUMBER;
use crate::arch::PAGE_SIZE;

const STACK_PAGE_NUM: usize = 64;

#[repr(align(4096))]
pub struct Stack {
    stack: [u8; PAGE_SIZE * STACK_PAGE_NUM],
}

impl Stack {
    pub fn top(&self) -> usize {
        (&self.stack as *const _ as usize) + PAGE_SIZE * STACK_PAGE_NUM
    }
}

const STACK: Stack = Stack {
    stack: [0; PAGE_SIZE * STACK_PAGE_NUM],
};

#[link_section = ".stack"]
static STACKS: [Stack; BOARD_CORE_NUMBER] = [STACK; BOARD_CORE_NUMBER];

#[no_mangle]
pub fn stack_of_core(core_id: usize) -> usize {
    STACKS[core_id].top()
}

#[no_mangle]
pub fn get_core_stack() -> usize {
    let core_id = crate::arch::Arch::core_id();
    STACKS[core_id].top()
}