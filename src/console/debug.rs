use core::arch::asm;
use core::fmt::{self, Write};

use crate::memory::virt_to_phys;
use crate::sbi::{sbi_ret, SbiResult};

#[macro_export]
macro_rules! dbg {
    () => {
        $crate::debug_println!("[{}:{}:{}]", core::file!(), core::line!(), core::column!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::debug_println!("[{}:{}:{}] {} = {:#?}",
                    core::file!(), core::line!(), core::column!(), core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::console::debug::_debug_print_args(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {{
        $crate::console::debug::_debug_println_args(core::format_args!($($arg)*));
    }};
}

pub fn _debug_print_args(args: fmt::Arguments) {
    let _ = write!(DebugConsole, "{args}");
}

pub fn _debug_println_args(args: fmt::Arguments) {
    let _ = writeln!(DebugConsole, "{args}");
}

pub struct DebugConsole;

impl fmt::Write for DebugConsole {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            let addr = virt_to_phys(s);
            match sbi_debug_console_write(addr as u64, s.len()) {
                Ok(n) => s = &s[n..],
                Err(_) => return Err(fmt::Error),
            }
        }

        Ok(())
    }
}

const DBCN_EID: u64 = 0x4442434E;

pub fn sbi_debug_console_write(start: u64, len: usize) -> SbiResult<usize> {
    let written_len;
    let status;

    unsafe {
        asm!(
            "ecall",
            in("a7") DBCN_EID,
            in("a6") 0,
            in("a0") len, // length of the buffer to print
            in("a1") start, // 64 lower bytes of the address
            in("a2") 0, // 64 upper bytes of the address
            lateout("a0") status,
            lateout("a1") written_len,
        );
    }

    sbi_ret(status, written_len)
}
