use alloc::fmt;
use core::arch::asm;
use core::ops;
use core::ops::Index;

use spinning_top::Spinlock as SpinLock;

use crate::arch::PAGE_SHIFT;
use crate::memory::{
    virt_to_phys, KERNEL_CODE_VIRTUAL, KERNEL_STACK_VIRTUAL, PHYSICAL_STACK_START, RAM_START,
    RAM_VIRTUAL_START,
};

// Page tables use the RSW bits of the first entry to denote if the page was allocated by the
// buddy allocator, or if it is a statically allocated (and it should not be unmapped!)

#[derive(Debug)]
pub enum PtError {
    AlreadyMappedLeaf,
    AlreadMappedIntermediate,
}

macro_rules! declare_flags {
     (flags {
         $($flag:ident : $value:expr),* $(,)?
     }) => {
         $(
             #[allow(unused)]
             pub const $flag: u8 = 1 << $value;
         )*

         const FLAGS: [(u8, &'static str); 8] = [
            $(($value, stringify!($flag)),)*
         ];
     };
 }

declare_flags! {
    flags {
        PTE_VALID: 0,
        PTE_READ: 1,
        PTE_WRITE: 2,
        PTE_EXECUTE: 3,
        PTE_USER: 4,
        PTE_GLOBAL: 5,
        PTE_ACCESSED: 6,
        PTE_DIRTY: 7,
    }
}

struct Flags(u8);

impl fmt::Debug for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = f.debug_set();

        for (flag, name) in FLAGS {
            if self.0 & (1 << flag) != 0 {
                flags.entry(&name);
            }
        }

        flags.finish()
    }
}

const STATIC_ALLOC: u64 = 1 << 9;

#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn new(ppn: u64, flags: u8) -> Self {
        Self::with_data(ppn, false, flags)
    }

    pub fn with_data(ppn: u64, data: bool, flags: u8) -> Self {
        Self(ppn << 10 | (data as u64 & 0b1) << 8 | (flags | PTE_VALID) as u64)
    }

    pub fn ppn(&self) -> u64 {
        self.0 >> 10
    }

    pub fn data(&self) -> bool {
        self.0 >> 8 & 0b1 == 1
    }

    pub fn flags(&self) -> u8 {
        (self.0 & 0b111) as u8
    }
}

struct Ppn(u64);

impl fmt::Debug for Ppn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("ppn", &Ppn(self.ppn()))
            .field("data", &self.data())
            .field("flags", &Flags(self.flags()))
            .finish()
    }
}

#[repr(align(4096))]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    fn ppn(&self) -> u64 {
        virt_to_phys(self) as u64 >> PAGE_SHIFT
    }

    fn set(&mut self, idx: u16, value: PageTableEntry) -> Result<(), PtError> {
        if self[idx].flags() & PTE_VALID != 0 {
            return Err(if self[idx].0 >> 1 & 0b111 == 0 {
                PtError::AlreadMappedIntermediate
            } else {
                PtError::AlreadyMappedLeaf
            });
        }

        self.0[idx as usize].0 |= value.0;

        Ok(())
    }
}

impl Index<u16> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl fmt::Debug for PageTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut pt = f.debug_list();

        for entry in &self.0 {
            if entry.flags() & PTE_VALID != 0 {
                pt.entry(&entry);
            }
        }

        pt.finish()
    }
}

#[derive(Debug)]
pub struct RootPageTable(PageTable);

impl ops::Deref for RootPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for RootPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

const EMPTY_PTE: PageTableEntry = PageTableEntry(0);

const EMPTY_STATIC_PT: PageTable = {
    let mut pt = PageTable([EMPTY_PTE; 512]);
    pt.0[0].0 |= STATIC_ALLOC;
    pt
};

fn vpn(addr: u64, idx: u8) -> u16 {
    ((addr >> (12 + 9 * idx)) & ((1 << 9) - 1)) as u16
}

/// Level 2 page table
/// SAFETY: must be modified with the lock for KERNEL_PAGE_TABLE
static mut HIGH_PT: PageTable = EMPTY_STATIC_PT;

/// Level 1 page table
/// SAFETY: must be modified with the lock for KERNEL_PAGE_TABLE
static mut STACK_PT: PageTable = EMPTY_STATIC_PT;

/// Level 3 page table
pub static KERNEL_PAGE_TABLE: SpinLock<RootPageTable> =
    SpinLock::new(RootPageTable(EMPTY_STATIC_PT));

pub fn init() {
    let mut root_pt = KERNEL_PAGE_TABLE.lock();
    let high_pt = unsafe { &mut *core::ptr::addr_of_mut!(HIGH_PT) };
    let stack_pt = unsafe { &mut *core::ptr::addr_of_mut!(STACK_PT) };

    let virtual_code_start = unsafe { &KERNEL_CODE_VIRTUAL as *const _ as u64 };
    let virtual_stack = unsafe { &KERNEL_STACK_VIRTUAL as *const _ as u64 };

    assert_eq!(
        vpn(virtual_code_start, 3),
        vpn(virtual_stack, 3),
        "Stack & code must share the same level 3"
    );

    root_pt
        .set(
            vpn(virtual_code_start, 3),
            PageTableEntry::new(high_pt.ppn(), PTE_VALID),
        )
        .unwrap();

    root_pt
        .set(
            vpn(RAM_VIRTUAL_START, 3),
            PageTableEntry::new(0, PTE_READ | PTE_WRITE),
        )
        .unwrap();

    high_pt
        .set(
            vpn(virtual_code_start, 2),
            PageTableEntry::new(
                RAM_START as u64 >> PAGE_SHIFT,
                PTE_EXECUTE | PTE_READ | PTE_WRITE,
            ),
        )
        .unwrap();

    high_pt
        .set(
            vpn(virtual_stack, 2),
            PageTableEntry::new(stack_pt.ppn(), 0),
        )
        .unwrap();

    stack_pt
        .set(
            vpn(virtual_stack, 1),
            PageTableEntry::new(
                PHYSICAL_STACK_START as u64 >> PAGE_SHIFT,
                PTE_READ | PTE_WRITE,
            ),
        )
        .unwrap();

    // crate::dbg!(&root_pt, &high_pt, &stack_pt);

    unsafe { asm!("csrw satp, {0}", in(reg) 9 << 60 | root_pt.ppn()) }
}
