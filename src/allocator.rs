// use core::mem::MaybeUninit;

use linked_list_allocator::LockedHeap;

use crate::boot::BootInfo;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(boot_info: &'static BootInfo) {
    let memory = boot_info.memory_region();

    unsafe {
        ALLOCATOR
            .lock()
            .init(memory.start.as_mut_ptr(), memory.size);
    }
}

// pub fn init_early_heap(early_heap: &'static mut [MaybeUninit<u8>]) {
//     ALLOCATOR.lock().init_from_slice(early_heap);
// }
