use divan::{AllocProfiler, Bencher};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

/// ```rust
/// fn fibo_500() -> i32 {
///     divan::black_box(fibo(500))
/// }
/// static __DIVAN_BENCH_FIBO_500: ::divan::__private::BenchEntry = {
///     {
///         #[used]
///         #[cfg_attr(windows, link_section = ".CRT$XCU")]
///         #[cfg_attr(
///             any(
///                 target_os = "android",
///                 target_os = "dragonfly",
///                 target_os = "freebsd",
///                 target_os = "fuchsia",
///                 target_os = "haiku",
///                 target_os = "illumos",
///                 target_os = "linux",
///                 target_os = "netbsd",
///                 target_os = "openbsd"
///             ),
///             link_section = ".init_array"
///         )]
///         #[cfg_attr(
///             any(
///                 target_os = "ios",
///                 target_os = "macos",
///                 target_os = "tvos",
///                 target_os = "watchos"
///             ),
///             link_section = "__DATA,__mod_init_func,mod_init_funcs"
///         )]
///         static PUSH: extern "C" fn() = push;
///         extern "C" fn push() {
///             static NODE: ::divan::__private::EntryList<::divan::__private::BenchEntry> =
///                 ::divan::__private::EntryList::new(&__DIVAN_BENCH_FIBO_500);
///             ::divan::__private::BENCH_ENTRIES.push(&NODE);
///         }
///     }
///     ::divan::__private::BenchEntry {
///         meta: ::divan::__private::EntryMeta {
///             raw_name: "fibo_500",
///             display_name: "fibo_500",
///             bench_options: ::std::option::Option::None,
///             module_path: ::std::module_path!(),
///             location: ::divan::__private::EntryLocation {
///                 file: ::std::file!(),
///                 line: 0u32,
///                 col: ::std::column!(),
///             },
///         },
///         bench: ::divan::__private::BenchEntryRunner::Plain(|divan| divan.bench(fibo_500)),
///     }
/// };
/// ```
#[divan::bench]
fn fibo_500() -> i32 {
    divan::black_box(1 + 1)
}

/// Functions that generate deterministic values.
mod gen {
    pub const LEN: usize = 100_000;

    pub fn rand_int_generator() -> impl FnMut() -> i32 {
        let mut rng = fastrand::Rng::with_seed(42);
        move || rng.i32(..)
    }

    pub fn rand_int_vec_generator() -> impl FnMut() -> Vec<i32> {
        let mut rand_int_generator = rand_int_generator();
        move || (0..LEN).map(|_| rand_int_generator()).collect()
    }

    pub fn sorted_int_vec_generator() -> impl FnMut() -> Vec<i32> {
        move || (0..LEN).map(|i| i as i32).collect()
    }
}

mod random {
    use super::*;

    #[divan::bench]
    /// ```rust
    /// static __DIVAN_BENCH_SORT: ::divan::__private::BenchEntry = {
    ///     {
    ///         #[used]
    ///         #[cfg_attr(windows, link_section = ".CRT$XCU")]
    ///         #[cfg_attr(
    ///             any(
    ///                 target_os = "android",
    ///                 target_os = "dragonfly",
    ///                 target_os = "freebsd",
    ///                 target_os = "fuchsia",
    ///                 target_os = "haiku",
    ///                 target_os = "illumos",
    ///                 target_os = "linux",
    ///                 target_os = "netbsd",
    ///                 target_os = "openbsd"
    ///             ),
    ///             link_section = ".init_array"
    ///         )]
    ///         #[cfg_attr(
    ///             any(
    ///                 target_os = "ios",
    ///                 target_os = "macos",
    ///                 target_os = "tvos",
    ///                 target_os = "watchos"
    ///             ),
    ///             link_section = "__DATA,__mod_init_func,mod_init_funcs"
    ///         )]
    ///         static PUSH: extern "C" fn() = push;
    ///         extern "C" fn push() {
    ///             static NODE: ::divan::__private::EntryList<::divan::__private::BenchEntry> =
    ///                 ::divan::__private::EntryList::new(&__DIVAN_BENCH_SORT);
    ///             ::divan::__private::BENCH_ENTRIES.push(&NODE);
    ///         }
    ///     }
    ///     ::divan::__private::BenchEntry {
    ///         meta: ::divan::__private::EntryMeta {
    ///             raw_name: "sort",
    ///             display_name: "sort",
    ///             bench_options: ::std::option::Option::None,
    ///             module_path: ::std::module_path!(),
    ///             location: ::divan::__private::EntryLocation {
    ///                 file: ::std::file!(),
    ///                 line: 0u32,
    ///                 col: ::std::column!(),
    ///             },
    ///         },
    ///         bench: ::divan::__private::BenchEntryRunner::Plain(sort),
    ///     }
    /// };
    /// ```
    fn sort(bencher: Bencher) {
        bencher
            .with_inputs(gen::rand_int_vec_generator())
            .bench_local_refs(|v| v.sort());
    }

    #[divan::bench]
    fn sort_unstable(bencher: Bencher) {
        bencher
            .with_inputs(gen::rand_int_vec_generator())
            .bench_local_refs(|v| v.sort_unstable());
    }
}

mod sorted {
    use super::*;

    #[divan::bench]
    fn sort(bencher: Bencher) {
        bencher
            .with_inputs(gen::sorted_int_vec_generator())
            .bench_local_refs(|v| v.sort());
    }

    #[divan::bench]
    fn sort_unstable(bencher: Bencher) {
        bencher
            .with_inputs(gen::sorted_int_vec_generator())
            .bench_local_refs(|v| v.sort_unstable());
    }
}
