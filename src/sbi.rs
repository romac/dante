use core::arch::asm;
use core::fmt;

pub type SbiResult<T> = Result<T, SbiError>;

#[allow(non_upper_case_globals)]
pub const SbiSuccess: isize = 0;

#[inline]
pub fn sbi_ret<T>(status: isize, value: T) -> SbiResult<T> {
    if status == SbiSuccess {
        Ok(value)
    } else {
        Err(SbiError::new(status))
    }
}

const SRST: usize = 0x53525354;

#[inline]
fn sbi_system_reset(reset_type: u32, reason: u32) -> ! {
    let status: isize;

    unsafe {
        asm!(
            "ecall",
            in("a7") SRST,
            in("a6") 0,
            in("a0") reset_type,
            in("a1") reason,
            lateout("a0") status,
            lateout("a1") _,
        )
    };

    panic!("Reset failed: {status}")
}

#[inline]
pub fn sbi_shutdown() -> ! {
    sbi_system_reset(0x00000000, 0x00000000)
}

#[inline]
pub fn sbi_panic() -> ! {
    sbi_system_reset(0x00000000, 0x00000001)
}

/// Error codes returned by SBI calls
///
/// note: `SBI_SUCCESS` is not represented here since this is to be used as the
/// error type in a `Result`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SbiError {
    /// The SBI call failed
    Failed,
    /// The SBI call is not implemented or the functionality is not available
    NotSupported,
    /// An invalid parameter was passed
    InvalidParameter,
    /// The SBI implementation has denied execution of the call functionality
    Denied,
    /// An invalid address was passed
    InvalidAddress,
    /// The resource is already available
    AlreadyAvailable,
    /// The resource was previously started
    AlreadyStarted,
    /// The resource was previously stopped
    AlreadyStopped,
    /// Unknowne error
    Unknown(isize),
}

impl SbiError {
    #[inline]
    pub fn new(n: isize) -> Self {
        match n {
            -1 => SbiError::Failed,
            -2 => SbiError::NotSupported,
            -3 => SbiError::InvalidParameter,
            -4 => SbiError::Denied,
            -5 => SbiError::InvalidAddress,
            -6 => SbiError::AlreadyAvailable,
            -7 => SbiError::AlreadyStarted,
            -8 => SbiError::AlreadyStopped,
            n => SbiError::Unknown(n),
        }
    }
}

impl fmt::Display for SbiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            SbiError::AlreadyAvailable => "resource is already available",
            SbiError::Denied => "SBI implementation denied execution",
            SbiError::Failed => "call to SBI failed",
            SbiError::InvalidAddress => "invalid address passed",
            SbiError::InvalidParameter => "invalid parameter passed",
            SbiError::NotSupported => "SBI call not implemented or functionality not available",
            SbiError::AlreadyStarted => "resource was already started",
            SbiError::AlreadyStopped => "resource was already stopped",
            SbiError::Unknown(n) => return write!(f, "unknown SBI error code: {n}"),
        };

        msg.fmt(f)
    }
}

