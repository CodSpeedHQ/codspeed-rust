use divan::black_box;
use std::collections::{BTreeMap, HashMap};

fn main() {
    divan::main();
}

#[divan::bench]
fn add() -> i32 {
    black_box(2) + black_box(1)
}

#[divan::bench]
#[ignore]
fn sub() -> i32 {
    black_box(2) - black_box(1)
}

#[divan::bench]
fn mul() -> i32 {
    black_box(2) * black_box(1)
}

#[divan::bench]
fn div() -> i32 {
    black_box(2) / black_box(1)
}

#[divan::bench]
fn rem() -> i32 {
    black_box(2) % black_box(1)
}

// 1, 1, 2, 3, 5, ...
mod fibonacci {
    use super::*;

    const VALUES: &[u64] = &[0, 5, 10, 20, 30];

    // O(n)
    #[divan::bench(args = VALUES)]
    fn iterative(n: u64) -> u64 {
        let mut previous = 1;
        let mut current = 1;

        for _ in 2..=n {
            let next = previous + current;
            previous = current;
            current = next;
        }

        current
    }

    // O(2^n)
    #[divan::bench(args = VALUES, max_time = 1)]
    fn recursive(n: u64) -> u64 {
        if n <= 1 {
            1
        } else {
            recursive(n - 2) + recursive(n - 1)
        }
    }

    #[allow(dead_code)]
    trait Map: Default {
        fn get(&self, key: u64) -> Option<u64>;
        fn set(&mut self, key: u64, value: u64);
    }

    impl Map for HashMap<u64, u64> {
        fn get(&self, key: u64) -> Option<u64> {
            self.get(&key).copied()
        }

        fn set(&mut self, key: u64, value: u64) {
            self.insert(key, value);
        }
    }

    impl Map for BTreeMap<u64, u64> {
        fn get(&self, key: u64) -> Option<u64> {
            self.get(&key).copied()
        }

        fn set(&mut self, key: u64, value: u64) {
            self.insert(key, value);
        }
    }

    // O(n)
    // ======================================
    // Recursive expansion of the bench macro
    // ======================================

    // #[cfg(not(any(
    //     windows,
    //     target_os = "android",
    //     target_os = "dragonfly",
    //     target_os = "freebsd",
    //     target_os = "fuchsia",
    //     target_os = "haiku",
    //     target_os = "illumos",
    //     target_os = "linux",
    //     target_os = "netbsd",
    //     target_os = "openbsd",
    //     target_os = "ios",
    //     target_os = "macos",
    //     target_os = "tvos",
    //     target_os = "watchos"
    // )))]
    // ::std::compile_error!("Unsupported target OS for `#[divan::bench]`");
    // static __DIVAN_BENCH_RECURSIVE_MEMOIZED: ::divan::__private::GroupEntry = {
    //     {
    //         #[used]
    //         #[cfg_attr(windows, link_section = ".CRT$XCU")]
    //         #[cfg_attr(
    //             any(
    //                 target_os = "android",
    //                 target_os = "dragonfly",
    //                 target_os = "freebsd",
    //                 target_os = "fuchsia",
    //                 target_os = "haiku",
    //                 target_os = "illumos",
    //                 target_os = "linux",
    //                 target_os = "netbsd",
    //                 target_os = "openbsd"
    //             ),
    //             link_section = ".init_array"
    //         )]
    //         #[cfg_attr(
    //             any(
    //                 target_os = "ios",
    //                 target_os = "macos",
    //                 target_os = "tvos",
    //                 target_os = "watchos"
    //             ),
    //             link_section = "__DATA,__mod_init_func,mod_init_funcs"
    //         )]
    //         static PUSH: extern "C" fn() = push;
    //         extern "C" fn push() {
    //             static NODE: ::divan::__private::EntryList<::divan::__private::GroupEntry> =
    //                 ::divan::__private::EntryList::new(&__DIVAN_BENCH_RECURSIVE_MEMOIZED);
    //             ::divan::__private::GROUP_ENTRIES.push(&NODE);
    //         }
    //     }
    //     static __DIVAN_ARGS: ::divan::__private::BenchArgs = ::divan::__private::BenchArgs::new();
    //     ::divan::__private::GroupEntry {
    //         meta: ::divan::__private::EntryMeta {
    //             raw_name: "recursive_memoized",
    //             display_name: "recursive_memoized",
    //             bench_options: ::std::option::Option::None,
    //             module_path: ::std::module_path!(),
    //             location: ::divan::__private::EntryLocation {
    //                 file: ::std::file!(),
    //                 line: 0u32,
    //                 col: ::std::column!(),
    //             },
    //         },
    //         generic_benches: ::std::option::Option::Some({
    //             &[&[
    //                 ::divan::__private::GenericBenchEntry {
    //                     group: &__DIVAN_BENCH_RECURSIVE_MEMOIZED,
    //                     bench: ::divan::__private::BenchEntryRunner::Args(|| {
    //                         __DIVAN_ARGS.runner(
    //                             || VALUES,
    //                             |arg| ::divan::__private::ToStringHelper(arg).to_string(),
    //                             |divan, __divan_arg| {
    //                                 divan.bench(|| {
    //                                     recursive_memoized::<BTreeMap<u64, u64>>(
    //                                         ::divan::__private::Arg::<u64>::get(__divan_arg),
    //                                     )
    //                                 })
    //                             },
    //                         )
    //                     }),
    //                     ty: ::std::option::Option::Some(::divan::__private::EntryType::new::<
    //                         BTreeMap<u64, u64>,
    //                     >()),
    //                     const_value: ::std::option::Option::None,
    //                 },
    //                 ::divan::__private::GenericBenchEntry {
    //                     group: &__DIVAN_BENCH_RECURSIVE_MEMOIZED,
    //                     bench: ::divan::__private::BenchEntryRunner::Args(|| {
    //                         __DIVAN_ARGS.runner(
    //                             || VALUES,
    //                             |arg| ::divan::__private::ToStringHelper(arg).to_string(),
    //                             |divan, __divan_arg| {
    //                                 divan.bench(|| {
    //                                     recursive_memoized::<HashMap<u64, u64>>(
    //                                         ::divan::__private::Arg::<u64>::get(__divan_arg),
    //                                     )
    //                                 })
    //                             },
    //                         )
    //                     }),
    //                     ty: ::std::option::Option::Some(::divan::__private::EntryType::new::<
    //                         HashMap<u64, u64>,
    //                     >()),
    //                     const_value: ::std::option::Option::None,
    //                 },
    //             ]]
    //         }),
    //     }
    // };
    #[divan::bench(
        types = [BTreeMap<u64, u64>, HashMap<u64, u64>],
        args = VALUES,
    )]
    fn recursive_memoized<M: Map>(n: u64) -> u64 {
        fn fibonacci<M: Map>(n: u64, cache: &mut M) -> u64 {
            if let Some(result) = cache.get(n) {
                return result;
            }

            if n <= 1 {
                return 1;
            }

            let result = fibonacci(n - 2, cache) + fibonacci(n - 1, cache);
            cache.set(n, result);
            result
        }

        fibonacci(n, &mut M::default())
    }
}
