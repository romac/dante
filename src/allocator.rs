use linked_list_allocator::LockedHeap;

use crate::boot::BootInfo;
use crate::prelude::*;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_kernel_heap(boot_info: &'static BootInfo) {
    let memory = boot_info.memory_region();

    unsafe {
        ALLOCATOR
            .lock()
            .init(memory.start.to_virt().as_mut_ptr(), memory.size);
    }
}

pub fn test_allocations() {
    let heap_value = Box::new(41);
    debug_println!("heap_value at {:p}", heap_value);
    debug_println!("heap_value: {}", *heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    debug_println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    debug_println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    debug_println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

    let string = String::from("crash");
    debug_println!("string at {:p}", string.as_str());

    debug_println!("It did not {string}!");
}
