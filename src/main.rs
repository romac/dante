#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

mod allocator;
mod arch;
mod boot;
mod console;
mod dtb;
mod memory;
mod page_table;
mod prelude;
mod sbi;

use core::arch::global_asm;
use core::panic::PanicInfo;

use boot::BootInfo;
use memory::PhysAddr;

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

    page_table::init_root_pt();
    debug_println!("\nPage table initialized");

    allocator::init_heap(boot_info);
    debug_println!("Heap initialized");

    // allocator::test_allocations();

    sbi::sbi_shutdown()
}
