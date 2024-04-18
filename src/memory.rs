use core::{fmt, ops};

// TODO: load these from symbols
const RAM_START: usize = 0x80000000;
const PHYSICAL_STACK_START: usize = 0x80000000 + 0x2000000 + 16 * 1024 * 1024;

extern "C" {
    #[link_name = "_KERNEL_CODE_VIRTUAL"]
    static KERNEL_CODE_VIRTUAL: u8;

    #[link_name = "_VIRTUAL_STACK"]
    static KERNEL_STACK_VIRTUAL: u8;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub const fn new(addr: usize) -> Self {
        VirtAddr(addr)
    }

    pub fn to_phys(self) -> PhysAddr {
        PhysAddr::new(virt_to_phys_addr(self.0))
    }

    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
}

impl ops::Add<usize> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: usize) -> VirtAddr {
        VirtAddr(self.0 + rhs)
    }
}

impl ops::Sub<usize> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, rhs: usize) -> VirtAddr {
        VirtAddr(self.0 - rhs)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtAddr(0x{:016x})", self.0)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl ops::Sub<VirtAddr> for VirtAddr {
    type Output = usize;

    fn sub(self, rhs: VirtAddr) -> usize {
        self.0 - rhs.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(usize);

impl PhysAddr {
    pub const fn new(addr: usize) -> Self {
        PhysAddr(addr)
    }

    pub fn to_virt(self) -> VirtAddr {
        VirtAddr::new(phys_to_virt_addr(self.0))
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.0 as *const u8
    }

    pub const fn as_mut_ptr(&self) -> *mut u8 {
        self.0 as *mut u8
    }
}

impl ops::Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> PhysAddr {
        PhysAddr(self.0 + rhs)
    }
}

impl ops::Sub<usize> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: usize) -> PhysAddr {
        PhysAddr(self.0 - rhs)
    }
}

impl ops::Sub<PhysAddr> for PhysAddr {
    type Output = usize;

    fn sub(self, rhs: PhysAddr) -> usize {
        self.0 - rhs.0
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PhysAddr(0x{:016x})", self.0)
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

pub fn virt_to_phys<T: ?Sized>(v: &T) -> usize {
    let addr = v as *const T as *const () as usize;
    virt_to_phys_addr(addr)
}

pub fn virt_to_phys_addr(addr: usize) -> usize {
    let virtual_code_start = unsafe { &KERNEL_CODE_VIRTUAL as *const _ as usize };
    let virtual_stack_start = unsafe { &KERNEL_STACK_VIRTUAL as *const _ as usize };

    if addr >= virtual_code_start {
        let offset = addr - virtual_code_start;
        RAM_START + offset
    } else if addr >= virtual_stack_start {
        let offset = addr - virtual_stack_start;
        PHYSICAL_STACK_START + offset
    } else {
        panic!("Unhandled virtual address: 0x{addr:x}")
    }
}

pub fn phys_to_virt_addr(addr: usize) -> usize {
    let virtual_code_start = unsafe { &KERNEL_CODE_VIRTUAL as *const _ as usize };
    let virtual_stack_start = unsafe { &KERNEL_STACK_VIRTUAL as *const _ as usize };

    if addr >= RAM_START {
        let offset = addr - RAM_START;
        virtual_code_start + offset
    } else if addr >= PHYSICAL_STACK_START {
        let offset = addr - PHYSICAL_STACK_START;
        virtual_stack_start + offset
    } else {
        panic!("Unhandled physical address: 0x{addr:x}")
    }
}
