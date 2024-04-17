// TODO: load these from symbols
const RAM_START: usize = 0x80000000;
const PHYSICAL_STACK_START: usize = 0x80000000 + 0x2000000 + 16 * 1024 * 1024;

extern "C" {
    #[link_name = "_KERNEL_CODE_VIRTUAL"]
    static KERNEL_CODE_VIRTUAL: u8;

    #[link_name = "_VIRTUAL_STACK"]
    static KERNEL_STACK_VIRTUAL: u8;
}

pub fn virt_to_phys<T: ?Sized>(v: &T) -> usize {
    let addr = v as *const T as *const () as usize;
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
