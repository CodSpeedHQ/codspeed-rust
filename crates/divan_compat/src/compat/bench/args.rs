use std::{any::TypeId, sync::OnceLock};

use super::Bencher;

/// Holds lazily-initialized runtime arguments to be passed into a benchmark.
///
/// `#[divan::bench]` stores this as a `__DIVAN_ARGS` global for each entry, and
/// then at runtime it is initialized once by a closure that creates the usable
/// `BenchArgsRunner`.
pub struct BenchArgs {
    args: OnceLock<ErasedArgsSlice>,
}

/// Type-erased `&'static [T]` that also stores names of the arguments.
struct ErasedArgsSlice {
    /// The start of `&[T]`.
    args: *const (),

    /// The start of `&[&'static str]`.
    names: *const &'static str,

    /// The number of arguments.
    len: usize,

    /// The ID of `T` to ensure correctness.
    arg_type: TypeId,
}

/// The result of making `BenchArgs` runnable from instantiating the arguments
/// list and providing a typed benchmarking implementation.
#[derive(Clone, Copy)]
pub struct BenchArgsRunner {
    args: &'static ErasedArgsSlice,
    bench: fn(Bencher, &ErasedArgsSlice, arg_index: usize),
}

impl BenchArgs {
    pub const fn new() -> Self {
        Self {
            args: OnceLock::new(),
        }
    }
}

impl Default for BenchArgs {
    fn default() -> Self {
        Self::new()
    }
}
