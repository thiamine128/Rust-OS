use core::{alloc::{GlobalAlloc, Layout}, borrow::BorrowMut, cmp::{max, min}, mem::*, ptr::NonNull};

use crate::{sync::cell::UPSafeCell, util::linked_list};

#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator<32> = HeapAllocator::<32>::empty();

pub struct Heap<const ORDER: usize> {
    free_list: [linked_list::LinkedList; ORDER],
    user: usize,
    allocated: usize,
    total: usize,
}

impl<const ORDER: usize> Heap<ORDER> {
    pub const fn new() -> Self {
        Heap {
            free_list: [linked_list::LinkedList::new(); ORDER],
            user: 0,
            allocated: 0,
            total: 0,
        }
    }

    pub const fn empty() -> Self {
        Self::new()
    }

    pub fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        end &= !size_of::<usize>() + 1;
        assert!(start <= end);

        let mut total = 0;
        let mut current_start = start;

        while current_start + size_of::<usize>() <= end {
            let lowbit = current_start & (!current_start + 1);
            let size = min(lowbit, prev_power_of_two(end - current_start));
            total += size;

            self.free_list[size.trailing_zeros() as usize].push(current_start as *mut usize);
            current_start += size;
        }

        self.total += total;
    }

    pub fn init(&mut self, start: usize, size: usize) {
        self.add_to_heap(start, start + size);
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;
        for i in class..self.free_list.len() {
            // Find the first non-empty size class
            if !self.free_list[i].is_empty() {
                // Split buffers
                for j in (class + 1..i + 1).rev() {
                    if let Some(block) = self.free_list[j].pop() {
                        self.free_list[j - 1]
                            .push((block as usize + (1 << (j - 1))) as *mut usize);
                        self.free_list[j - 1].push(block);
                    } else {
                        return Err(());
                    }
                }

                let result = NonNull::new(
                    self.free_list[class]
                        .pop()
                        .expect("current block should have free space now")
                        as *mut u8,
                );
                if let Some(result) = result {
                    self.user += layout.size();
                    self.allocated += size;
                    return Ok(result);
                } else {
                    return Err(());
                }
            }
        }
        Err(())
    }

    /// Dealloc a range of memory from the heap
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;
        // Put back into free list
        self.free_list[class].push(ptr.as_ptr() as *mut usize);

        // Merge free buddy lists
        let mut current_ptr = ptr.as_ptr() as usize;
        let mut current_class = class;

        while current_class < self.free_list.len() - 1 {
            let buddy = current_ptr ^ (1 << current_class);
            let mut flag = false;
            for block in self.free_list[current_class].iter_mut() {
                if block.value() as usize == buddy {
                    block.pop();
                    flag = true;
                    break;
                }
            }

            // Free buddy found
            if flag {
                self.free_list[current_class].pop();
                current_ptr = min(current_ptr, buddy);
                current_class += 1;
                self.free_list[current_class].push(current_ptr as *mut usize);
            } else {
                break;
            }
        }

        self.user -= layout.size();
        self.allocated -= size;
    }

    /// Return the number of bytes that user requests
    pub fn stats_alloc_user(&self) -> usize {
        self.user
    }

    /// Return the number of bytes that are actually allocated
    pub fn stats_alloc_actual(&self) -> usize {
        self.allocated
    }

    /// Return the total number of bytes in the heap
    pub fn stats_total_bytes(&self) -> usize {
        self.total
    }
}

pub struct HeapAllocator<const ORDER: usize>(UPSafeCell<Heap<ORDER>>);

fn prev_power_of_two(num: usize) -> usize {
    1 << (usize::BITS as usize - num.leading_zeros() as usize - 1)
}

const KERNEL_HEAP_SIZE: usize = 0x100000;
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// initialize kernel heap.
pub fn init_heap() {
    let start = unsafe {HEAP_SPACE.as_ptr()} as usize;
    HEAP_ALLOCATOR.init(start, KERNEL_HEAP_SIZE);
}

impl<const ORDER: usize> HeapAllocator<ORDER> {
    pub const fn empty() -> Self {
        Self(UPSafeCell::new(Heap::<ORDER>::empty()))
    }

    pub fn init(&self, start: usize, size: usize) {
        self.0.borrow_mut().init(start, size);
    }
}

unsafe impl<const ORDER: usize> GlobalAlloc for HeapAllocator<ORDER> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
       self.0.borrow_mut().alloc(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.borrow_mut().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

/// handle heap allocation error/
#[alloc_error_handler]
pub fn handle_heap_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout={:?}", layout);
}