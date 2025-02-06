use crate::__private::EntryMeta;

pub(crate) fn generate(
    bench_display_name: impl std::fmt::Display,
    bench_meta: &EntryMeta,
) -> String {
    let file = bench_meta.location.file;
    let mut module_path = bench_meta
        .module_path_components()
        .skip(1)
        .collect::<Vec<_>>()
        .join("::");
    if !module_path.is_empty() {
        module_path.push_str("::");
    }
    let uri = format!("{file}::{module_path}{bench_display_name}");

    uri
}

pub(crate) fn append_arg(uri: &str, arg: &str) -> String {
    format!("{uri}[{arg}]")
}
