use std::ffi::CString;
use std::sync::OnceLock;

mod ffi;

pub struct InstrumentHooks(*mut ffi::InstrumentHooks);

unsafe impl Send for InstrumentHooks {}
unsafe impl Sync for InstrumentHooks {}

impl InstrumentHooks {
    #[inline(always)]
    pub fn new() -> Option<Self> {
        let ptr = unsafe { ffi::instrument_hooks_init() };
        if ptr.is_null() {
            None
        } else {
            Some(InstrumentHooks(ptr))
        }
    }

    /// Returns a singleton instance of `InstrumentHooks`.
    #[inline(always)]
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<InstrumentHooks> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let instance = InstrumentHooks::new().expect("Failed to initialize InstrumentHooks");
            instance
                .set_integration("codspeed-rust", env!("CARGO_PKG_VERSION"))
                .expect("Failed to set integration");
            instance
        })
    }

    #[inline(always)]
    pub fn is_instrumented(&self) -> bool {
        unsafe { ffi::instrument_hooks_is_instrumented(self.0) }
    }

    #[inline(always)]
    pub fn start_benchmark(&self) -> Result<(), i8> {
        let result = unsafe { ffi::instrument_hooks_start_benchmark(self.0) };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    #[inline(always)]
    pub fn stop_benchmark(&self) -> Result<(), i8> {
        let result = unsafe { ffi::instrument_hooks_stop_benchmark(self.0) };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    #[inline(always)]
    pub fn set_executed_benchmark(&self, pid: i32, uri: &str) -> Result<(), i8> {
        let c_uri = CString::new(uri).map_err(|_| -1i8)?;
        let result =
            unsafe { ffi::instrument_hooks_set_executed_benchmark(self.0, pid, c_uri.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    #[inline(always)]
    pub fn set_integration(&self, name: &str, version: &str) -> Result<(), i8> {
        let c_name = CString::new(name).map_err(|_| -1i8)?;
        let c_version = CString::new(version).map_err(|_| -1i8)?;
        let result = unsafe {
            ffi::instrument_hooks_set_integration(self.0, c_name.as_ptr(), c_version.as_ptr())
        };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn start_callgrind_instrumentation() -> Result<(), i8> {
        let result = unsafe { ffi::callgrind_start_instrumentation() };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn stop_callgrind_instrumentation() -> Result<(), i8> {
        let result = unsafe { ffi::callgrind_stop_instrumentation() };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn set_feature(feature: ffi::instrument_hooks_feature_t, enabled: bool) {
        unsafe { ffi::instrument_hooks_set_feature(feature, enabled) };
    }
}

impl Drop for InstrumentHooks {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { ffi::instrument_hooks_deinit(self.0) };
            self.0 = std::ptr::null_mut();
        }
    }
}
