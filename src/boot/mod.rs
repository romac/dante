use core::fmt;
use fdt::Fdt;

use crate::memory::{PhysAddr, RAM_PHYS_START};

static mut BOOT_INFO: Option<BootInfo> = None;

#[derive(Debug)]
pub struct BootInfo {
    pub hart_id: usize,
    pub dtb_addr: PhysAddr,
    pub fdt: Fdt<'static>,
}

impl BootInfo {
    pub fn new(hart_id: usize, dtb_addr: PhysAddr) -> &'static Self {
        let fdt = unsafe { Fdt::from_ptr(dtb_addr.to_virt().as_ptr()).unwrap() };

        let boot_info = Self {
            hart_id,
            dtb_addr,
            fdt,
        };

        unsafe {
            BOOT_INFO = Some(boot_info);
            BOOT_INFO.as_ref().unwrap()
        }
    }

    pub fn memory_region(&self) -> Region {
        let memory = Region::from(self.fdt.memory().regions().next().unwrap());
        let start = PhysAddr::new(RAM_PHYS_START);
        let size = memory.start.as_usize() - (RAM_PHYS_START - memory.start.as_usize());
        let max_size = usize::MAX - start.to_virt().as_usize();

        Region {
            start,
            size: core::cmp::min(size, max_size),
        }

        // let reserved_memory = self.fdt.find_node("/reserved-memory");
        // let last_reserved_memory_region = reserved_memory
        //     .map(|node| node.children().flat_map(|node| node.reg().unwrap()))
        //     .map(|regions| regions.map(Region::from))
        //     .and_then(|regions| regions.max_by_key(|reg| reg.end()));
        //
        // if let Some(last_reserved_memory_region) = last_reserved_memory_region {
        //     let reserved_size = last_reserved_memory_region.end() - memory_region.start;
        //
        //     Region::new(
        //         last_reserved_memory_region.end(),
        //         memory_region.size - reserved_size,
        //     )
        // } else {
        //     memory_region
        // }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Region {
    pub start: PhysAddr,
    pub size: usize,
}

impl Region {
    pub fn new(start: PhysAddr, size: usize) -> Self {
        Self { start, size }
    }

    pub fn end(&self) -> PhysAddr {
        self.start + self.size
    }
}

impl From<fdt::standard_nodes::MemoryRegion> for Region {
    fn from(region: fdt::standard_nodes::MemoryRegion) -> Self {
        Self {
            start: PhysAddr::new(region.starting_address as usize),
            size: region.size.unwrap(),
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} ({} bytes)", self.start, self.end(), self.size)
    }
}
