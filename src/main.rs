#![no_std]
#![no_main]
#![allow(clippy::missing_safety_doc)]

mod console;
mod memory;
mod sbi;

use core::arch::global_asm;
use core::panic::PanicInfo;

use crate::memory::phys_to_virt_addr;

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

// // 1MB of early heap
// const EARLY_HEAP_LEN: usize = 1024 * 1024;

// global_asm!(
//     ".section .data",
//     ".global _early_heap",
//     ".align 12",
//     "_early_heap:",
//     ".rep 1024 * 1024",
//     "    .byte 0",
//     ".endr"
// );
//
// extern "C" {
//     #[link_name = "_early_heap"]
//     static mut EARLY_HEAP: u8;
// }

#[export_name = "_kmain"]
pub unsafe extern "C" fn kmain(hart_id: usize, phys_dtb: usize) -> ! {
    // let _early_heap = unsafe {
    //     core::slice::from_raw_parts_mut(
    //         addr_of_mut!(EARLY_HEAP) as *mut MaybeUninit<u8>,
    //         EARLY_HEAP_LEN,
    //     )
    // };

    debug_println!("{BANNER}");

    let virt_dtb = phys_to_virt_addr(phys_dtb);

    debug_println!("Kernel arguments:");
    debug_println!("  HART:       {hart_id}");
    debug_println!("  DeviceTree:");
    debug_println!("    Physical: 0x{phys_dtb:x}");
    debug_println!("    Virtual:  0x{virt_dtb:x}");

    debug_dtb(virt_dtb);

    loop {
        wfi();
    }
}

fn debug_dtb(virt_dtb: usize) {
    let fdt = unsafe { fdt::Fdt::from_ptr(virt_dtb as *const u8).unwrap() };
    // dbg!(&fdt);

    debug_println!("\nDeviceTree:");
    debug_println!("  Model:               {}", fdt.root().model());
    debug_println!("  Compatible with:     {}", fdt.root().compatible().first());
    debug_println!("  CPUs:                {}", fdt.cpus().count());
    debug_println!("  Memory regions:      {}", fdt.memory().regions().count());

    let memory = fdt.memory().regions().next().unwrap();

    debug_println!(
        "  Memory:              {:#X} - {:#X} ({} bytes)",
        memory.starting_address as usize,
        memory.starting_address as usize + memory.size.unwrap(),
        memory.size.unwrap()
    );

    if let Some(reserved_memory) = fdt.find_node("/reserved-memory") {
        debug_println!(
            "  Reserved regions:    {}",
            reserved_memory.children().count()
        );

        for (i, region) in reserved_memory.children().enumerate() {
            let reg = region.reg().unwrap().next().unwrap();
            debug_println!(
                "  Reserved memory #{i}:  {:#X} - {:#X} ({} bytes)",
                reg.starting_address as usize,
                reg.starting_address as usize + reg.size.unwrap(),
                reg.size.unwrap()
            );
        }
    }

    let chosen = fdt.chosen();
    if let Some(bootargs) = chosen.bootargs() {
        debug_println!("  Boot arguments:      {:?}", bootargs);
    }

    if let Some(stdout) = chosen.stdout() {
        debug_println!("  Stdout device:       {}", stdout.name);
    }

    let soc = fdt.find_node("/soc");
    debug_println!(
        "  Has SoC?             {}",
        if soc.is_some() { "yes" } else { "no" }
    );
    if let Some(soc) = soc {
        debug_print!("  SoC Children:");
        for (i, child) in soc.children().enumerate() {
            if i == 0 {
                debug_println!("        {}", child.name);
            } else {
                debug_println!("                       {}", child.name);
            }
        }
    }
}
