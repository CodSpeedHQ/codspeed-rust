use super::AnyBenchEntry;

/// Generate the codspeed URI for a benchmark entry.
/// The format is `"{file}::{module_path}::{bench_name}"`.
///
/// # Bench Name Computation
/// There are three elements to consider:
/// - The static entry name from metadata, i.e., the name of the benchmarked function.
/// - The type (since benchmarks can be generic).
/// - The arguments (as you can statically specify a list of inputs for a benchmark).
///
/// Depending on the nesting, you need to check three places:
/// - `entry.meta().display_name`
/// - `entry.display_name()`
/// - `closure_bench_display_name`, computed by divan when calling the closure that runs the bench
///
/// From these three elements, we derive the codspeed bench name `function_name[type?, arg?]`:
/// - In the simple case (no generic, no args via macro), all three are equivalent.
/// - With an arg and no type, the first two are equal to the function name.
/// - With no arg and a type, the last two are equal to the type name.
/// - With both an arg and a type, all three have distinct values: the function name, the arg, and the type, respectively.
pub(crate) fn generate(bench_entry: &AnyBenchEntry, closure_bench_display_name: &str) -> String {
    let bench_function_name = bench_entry.meta().display_name;

    let (bench_type_name, bench_arg_name) = {
        let bench_function_or_type_name = bench_entry.display_name().to_string();

        let type_name = if bench_function_or_type_name == bench_function_name {
            None
        } else {
            Some(bench_function_or_type_name)
        };

        let arg_name = match type_name.as_ref() {
            None => {
                if closure_bench_display_name == bench_function_name {
                    None
                } else {
                    Some(closure_bench_display_name)
                }
            }
            Some(type_name) => {
                if closure_bench_display_name == type_name {
                    None
                } else {
                    Some(closure_bench_display_name)
                }
            }
        };

        (type_name, arg_name)
    };

    let mut bench_name = bench_function_name.to_string();

    match (bench_type_name, bench_arg_name) {
        (None, None) => {}
        (Some(type_name), None) => {
            bench_name.push_str(format!("[{type_name}]").as_str());
        }
        (None, Some(arg_name)) => {
            bench_name.push_str(format!("[{arg_name}]").as_str());
        }
        (Some(type_name), Some(arg_name)) => {
            bench_name.push_str(format!("[{type_name}, {arg_name}]").as_str());
        }
    }

    let file = bench_entry.meta().location.file;
    // In the context of a bench, the top level module will be a repetition of the file name, we
    // chose to skip it
    let mut module_path = bench_entry
        .meta()
        .module_path_components()
        .skip(1)
        .collect::<Vec<_>>()
        .join("::");
    if !module_path.is_empty() {
        module_path.push_str("::");
    }
    let uri = format!("{file}::{module_path}{bench_name}");

    uri
}

#[cfg(test)]
mod tests {
    use crate::__private::*;

    use super::*;

    #[test]
    fn test_generate_simple_case() {
        let meta = EntryMeta {
            display_name: "bench_function",
            raw_name: "bench_function",
            module_path: "test::module",
            location: EntryLocation {
                file: "foo.rs",
                ..Default::default()
            },
            bench_options: None,
        };
        let bench_entry = BenchEntry {
            meta,
            bench: BenchEntryRunner::Plain(|_| {}),
        };
        let closure_bench_display_name = "bench_function";
        let uri = generate(
            &AnyBenchEntry::Bench(&bench_entry),
            closure_bench_display_name,
        );
        assert_eq!(uri, "foo.rs::module::bench_function");
    }

    #[test]
    fn test_generate_with_arg() {
        let meta = EntryMeta {
            display_name: "bench_function",
            raw_name: "bench_function",
            module_path: "test::module",
            location: EntryLocation {
                file: "foo.rs",
                ..Default::default()
            },
            bench_options: None,
        };
        let bench_entry = BenchEntry {
            meta,
            bench: BenchEntryRunner::Plain(|_| {}),
        };
        let closure_bench_display_name = "ArgName";
        let uri = generate(
            &AnyBenchEntry::Bench(&bench_entry),
            closure_bench_display_name,
        );
        assert_eq!(uri, "foo.rs::module::bench_function[ArgName]");
    }

    #[test]
    fn test_generate_no_module_path() {
        let meta = EntryMeta {
            display_name: "bench_function",
            raw_name: "bench_function",
            module_path: "test",
            location: EntryLocation {
                file: "bar.rs",
                ..Default::default()
            },
            bench_options: None,
        };
        let bench_entry = BenchEntry {
            meta,
            bench: BenchEntryRunner::Plain(|_| {}),
        };
        let closure_bench_display_name = "bench_function";
        let uri = generate(
            &AnyBenchEntry::Bench(&bench_entry),
            closure_bench_display_name,
        );
        assert_eq!(uri, "bar.rs::bench_function");
    }

    #[allow(non_upper_case_globals)]
    static mock_group_entry: GroupEntry = GroupEntry {
        meta: EntryMeta {
            display_name: "bench_function",
            raw_name: "bench_function",
            module_path: "test::module",
            location: EntryLocation {
                file: "main.rs",
                line: 0,
                col: 0,
            },
            bench_options: None,
        },
        generic_benches: None,
    };
    #[test]
    fn test_generate_with_type() {
        // Without arg
        let hashmap_bench_entry = GenericBenchEntry {
            group: &mock_group_entry,
            bench: BenchEntryRunner::Plain(|_| {}),
            ty: Some(EntryType::new::<std::collections::HashMap<&str, f64>>()),
            const_value: None,
        };
        let entry = AnyBenchEntry::GenericBench(&hashmap_bench_entry);
        let uri = generate(&entry, entry.display_name());
        assert_eq!(uri, "main.rs::module::bench_function[HashMap<&str, f64>]");
    }

    #[test]
    fn test_generate_with_type_and_arg() {
        let vec_bench_entry = GenericBenchEntry {
            group: &mock_group_entry,
            bench: BenchEntryRunner::Plain(|_| {}),
            ty: Some(EntryType::new::<Vec<f64>>()),
            const_value: None,
        };

        let closure_bench_display_name = "ArgName";
        let uri = generate(
            &AnyBenchEntry::GenericBench(&vec_bench_entry),
            closure_bench_display_name,
        );
        assert_eq!(uri, "main.rs::module::bench_function[Vec<f64>, ArgName]");
    }
}
