use core::alloc::Layout;

use buddy_system_allocator::LockedHeap;


#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

const KERNEL_HEAP_SIZE: usize = 0x100000;
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// initialize kernel heap.
pub fn init_heap() {
    let mut heap_allocator = HEAP_ALLOCATOR.lock();
    let start = unsafe {HEAP_SPACE.as_ptr()} as usize;
    unsafe {heap_allocator.init(start, KERNEL_HEAP_SIZE)};
}

/// handle heap allocation error/
#[alloc_error_handler]
pub fn handle_heap_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout={:?}", layout);
}