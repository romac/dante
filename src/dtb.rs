use fdt::Fdt;

use crate::{debug_print, debug_println};

pub fn debug_dtb(fdt: &Fdt<'_>) {
    // dbg!(&fdt);

    debug_println!("\nDeviceTree:");
    debug_println!("  Model:               {}", fdt.root().model());
    debug_println!("  Compatible with:     {}", fdt.root().compatible().first());
    debug_println!("  CPUs:                {}", fdt.cpus().count());
    debug_println!("  Memory regions:      {}", fdt.memory().regions().count());

    let memory = fdt.memory().regions().next().unwrap();

    debug_println!(
        "  Memory:              {:#x} - {:#X} ({} bytes)",
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
                "  Reserved memory #{i}:  {:#x} - {:#x} ({} bytes)",
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
