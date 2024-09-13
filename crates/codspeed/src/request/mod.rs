const CG_BASE: u32 = ((b'C' as u32) << 24) + ((b'T' as u32) << 16);

#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum ClientRequest {
    RunningOnValgrind = 0x1001,
    ZeroStatistics = CG_BASE + 1,
    DumpStatisticsAt = CG_BASE + 3,
    StartInstrumentation = CG_BASE + 4,
    StopInstrumentation = CG_BASE + 5,
}

pub use self::arch::{send_client_request, Value};

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64.rs"]
mod arch;

#[cfg(target_arch = "x86")]
#[path = "arch/x86.rs"]
mod arch;

#[cfg(target_arch = "arm")]
#[path = "arch/arm.rs"]
mod arch;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64.rs"]
mod arch;

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "arm",
    target_arch = "aarch64"
)))]
#[path = "arch/unsupported.rs"]
mod arch;
