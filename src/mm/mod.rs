pub mod address;
pub mod allocator;
pub mod config;
pub mod frame_allocator;
pub mod heap;
pub mod interface;
pub mod page_allocator;
pub mod paging;
pub mod stack;

pub use allocator::*;
pub use self::page_allocator::Page;
pub use self::frame_allocator::Frame;

// Only run on core 0.
pub fn allocator_init() {
    match frame_allocator::init() {
        Ok(_) => {
            info!("frame allocator init ok");
            frame_allocator::dump_frame_allocator_state();
        }
        Err(e) => {
            warn!("frame allocator init failed, error {}", e);
        }
    }
    info!("frame allocator init ok");
    match page_allocator::init() {
        Ok(_) => {
            info!("page allocator init ok");
            page_allocator::dump_page_allocator_state();
        }
        Err(e) => {
            warn!("page allocator init failed, error {}", e);
        }
    }
    info!("page_allocator init ok");
    
}

#[cfg(feature = "terminal")]
pub fn dump_mm_usage() {
    println!("--------------- HEAP Memory ---------------");
    heap::dump_heap_allocator_state();
    println!("------------- Virtual Address -------------");
    page_allocator::dump_page_allocator_state();
    println!("------------ Physical Address -------------");
    frame_allocator::dump_frame_allocator_state();
}