use std::ffi::CString;

use crate::request::{send_client_request, ClientRequest, Value};

#[inline(always)]
pub fn is_instrumented() -> bool {
    let valgrind_depth = unsafe {
        send_client_request(
            0,
            &[ClientRequest::RunningOnValgrind as Value, 0, 0, 0, 0, 0],
        )
    };
    valgrind_depth > 0
}

#[inline(always)]
pub fn set_metadata() {
    let full_metadata = CString::new(format!(
        "Metadata: codspeed-rust {}",
        env!("CARGO_PKG_VERSION")
    ))
    .expect("CString::new failed");
    unsafe {
        send_client_request(
            0,
            &[
                ClientRequest::DumpStatisticsAt as Value,
                full_metadata.as_ptr() as Value,
                0,
                0,
                0,
                0,
            ],
        );
    }
}

/// Switches Callgrind instrumentation on. With `--cache-sim=yes` this flushes the
/// simulated cache, so it is issued once before the warmup and never between the
/// warmup and the measured iteration.
#[inline(always)]
pub fn start_instrumentation() {
    unsafe {
        send_client_request(
            0,
            &[ClientRequest::StartInstrumentation as Value, 0, 0, 0, 0, 0],
        );
    }
}

/// Zeroes the cost centers without touching the simulated cache state.
#[inline(always)]
pub fn zero_statistics() {
    unsafe {
        send_client_request(0, &[ClientRequest::ZeroStatistics as Value, 0, 0, 0, 0, 0]);
    }
}

#[inline(always)]
pub fn stop(name: &CString) {
    unsafe {
        send_client_request(
            0,
            &[ClientRequest::StopInstrumentation as Value, 0, 0, 0, 0, 0],
        );
        send_client_request(
            0,
            &[
                ClientRequest::DumpStatisticsAt as Value,
                name.as_ptr() as Value,
                0,
                0,
                0,
                0,
            ],
        );
    }
}
