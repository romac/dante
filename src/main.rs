#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

mod allocator;
mod boot;
mod console;
mod dtb;
mod memory;
mod prelude;
mod sbi;

use core::arch::global_asm;
use core::panic::PanicInfo;

use boot::BootInfo;
use memory::PhysAddr;

use prelude::*;

pub const BANNER: &str = r#"
      ___           ___           ___           ___           ___     
     /\  \         /\  \         /\__\         /\  \         /\  \    
    /::\  \       /::\  \       /::|  |        \:\  \       /::\  \   
   /:/\:\  \     /:/\:\  \     /:|:|  |         \:\  \     /:/\:\  \  
  /:/  \:\__\   /::\~\:\  \   /:/|:|  |__       /::\  \   /::\~\:\  \ 
 /:/__/ \:|__| /:/\:\ \:\__\ /:/ |:| /\__\     /:/\:\__\ /:/\:\ \:\__\
 \:\  \ /:/  / \/__\:\/:/  / \/__|:|/:/  /    /:/  \/__/ \:\~\:\ \/__/
  \:\  /:/  /       \::/  /      |:/:/  /    /:/  /       \:\ \:\__\  
   \:\/:/  /        /:/  /       |::/  /     \/__/         \:\ \/__/  
    \::/__/        /:/  /        /:/  /                     \:\__\    
     --            \/__/         \/__/                       \/__/    
"#;

// Wait for interrupt, allows the CPU to go into a power saving mode
#[inline]
pub fn wfi() {
    unsafe { core::arch::asm!("wfi") }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("\n==== PANIC ====\n{info}");

    sbi::sbi_panic();
}

global_asm!(include_str!("boot/boot.s"));

#[export_name = "_kmain"]
pub unsafe extern "C" fn kmain(hart_id: usize, phys_dtb: usize) -> ! {
    let phys_dtb = PhysAddr::new(phys_dtb);
    let boot_info = BootInfo::new(hart_id, phys_dtb);

    kernel_main(boot_info)
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    debug_println!("{BANNER}");
    debug_println!("Kernel arguments:");
    debug_println!("  HART: {}", boot_info.hart_id);
    debug_println!("  DeviceTree:");
    debug_println!("    Physical: {}", boot_info.dtb_addr);
    debug_println!("    Virtual:  {}", boot_info.dtb_addr.to_virt());

    let memory_region = boot_info.memory_region();

    debug_println!("  Memory region:");
    debug_println!("    Start: {}", memory_region.start);
    debug_println!("    End:   {}", memory_region.end());
    debug_println!("    Size:  {} bytes", memory_region.size);

    // dtb::debug_dtb(&boot_info.fdt);

    allocator::init_heap(boot_info);

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

    sbi::sbi_shutdown()
}
