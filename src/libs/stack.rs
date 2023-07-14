use crate::arch::PAGE_SIZE;
use crate::board::BOARD_CORE_NUMBER;

const STACK_PAGE_NUM: usize = 10;

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

/// Todo: this is not safe.
/// We need to find a way of monitoring kernel stack overflow.
#[link_section = ".stack"]
static STACKS: [Stack; BOARD_CORE_NUMBER] = [STACK; BOARD_CORE_NUMBER];

#[no_mangle]
pub fn stack_of_core(core_id: usize) -> usize {
    STACKS[core_id].top()
}
