#![no_std]
#![no_main]
#![allow(clippy::missing_safety_doc)]

mod console;
mod memory;
mod sbi;

use core::arch::global_asm;
use core::panic::PanicInfo;

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
    debug_println!("{BANNER}");

    debug_println!("Kernel arguments:");
    debug_println!("  hart: {hart_id}");
    debug_println!("  dtb (physical): 0x{phys_dtb:x}");

    debug_dtb();

    loop {
        wfi();
    }
}

static MY_FDT: &[u8] = include_bytes!("../virt.dtb");

fn debug_dtb() {
    let fdt = fdt::Fdt::new(MY_FDT).unwrap();

    debug_println!("\nDeviceTree:");
    debug_println!("  Model:               {}", fdt.root().model());
    debug_println!("  Compatible with:     {}", fdt.root().compatible().first());
    debug_println!("  CPUs:                {}", fdt.cpus().count());
    debug_println!(
        "  First memory region: {:#X}",
        fdt.memory().regions().next().unwrap().starting_address as usize
    );

    let chosen = fdt.chosen();
    if let Some(bootargs) = chosen.bootargs() {
        debug_println!("  Boot arguments:      {:?}", bootargs);
    }

    if let Some(stdout) = chosen.stdout() {
        debug_println!("  Stdout device:       {}", stdout.name);
    }

    let soc = fdt.find_node("/soc");
    debug_println!(
        "  Has /soc node?       {}",
        if soc.is_some() { "yes" } else { "no" }
    );
    if let Some(soc) = soc {
        debug_println!("  Children:");
        for child in soc.children() {
            debug_println!("    {}", child.name);
        }
    }
}
