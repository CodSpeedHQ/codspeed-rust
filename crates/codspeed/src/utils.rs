use std::io;
use std::path::{Component, Path, PathBuf};

fn get_parent_git_repo_path(abs_path: &Path) -> io::Result<PathBuf> {
    if abs_path.join(".git").exists() {
        Ok(abs_path.to_path_buf())
    } else {
        get_parent_git_repo_path(
            abs_path
                .parent()
                .ok_or(io::Error::from(io::ErrorKind::NotFound))?,
        )
    }
}

/// Returns `abs_path` relative to the root of its git repository, for the benchmark URI.
///
/// It canonicalizes the path, which resolves the symlinks and the `.` and `..` segments.
/// When the file can't be canonicalized (e.g. it no longer exists), it removes the `.` and
/// `..` segments without touching the file system. A URL parser would resolve them in the
/// URI, and break the benchmark page and its flame graph.
///
/// Outside of a git repository, it returns the whole path.
pub fn get_git_relative_path<P>(abs_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let abs_path = abs_path.as_ref();
    let path = abs_path
        .canonicalize()
        .unwrap_or_else(|_| normalize_path(abs_path));
    match get_parent_git_repo_path(&path) {
        // `repo_path` is an ancestor of `path`, so the prefix always matches
        Ok(repo_path) => path
            .strip_prefix(repo_path)
            .expect("Repository path is malformed.")
            .to_path_buf(),
        Err(_) => path,
    }
}

/// Returns the path of a `file!()` path relative to the root of its git repository, for the
/// benchmark URI.
///
/// This makes the URI independent of where the Cargo workspace sits in the repository.
pub fn get_git_relative_file_path(file: &str) -> String {
    get_git_relative_path(workspace_file_path(file))
        .to_string_lossy()
        .into_owned()
}

/// Returns the path on disk of a `file!()` path.
///
/// `file!()` is relative to the Cargo workspace root, which cargo-codspeed exports in
/// `CODSPEED_CARGO_WORKSPACE_ROOT`. Without this variable, it returns `file` as is.
fn workspace_file_path(file: &str) -> PathBuf {
    match std::env::var_os("CODSPEED_CARGO_WORKSPACE_ROOT") {
        Some(workspace_root) => PathBuf::from(workspace_root).join(file),
        None => PathBuf::from(file),
    }
}

/// Builds a benchmark URI by joining the non-empty segments with `::`.
///
/// Segments can be empty when `criterion_group!`/`criterion_main!` are bypassed (custom main):
/// `current_file` and `macro_group` are not set in that case and must not produce empty
/// `::` parts (e.g. `::::my_group::my_bench`).
pub fn build_uri(segments: &[&str]) -> String {
    segments
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("::")
}

/// Resolves the caller's source file path for URI generation, anchored to the
/// workspace root when running under the CodSpeed runner.
///
/// `#[track_caller]` so the location resolves to the original benchmark call site
/// rather than this helper.
#[track_caller]
pub fn caller_file_path() -> String {
    workspace_file_path(std::panic::Location::caller().file())
        .to_string_lossy()
        .into_owned()
}

/// Removes the `.` and `..` segments of a path, without touching the file system.
///
/// A `..` segment removes the segment before it, so leading `..` segments stay.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components: Vec<Component> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match components.last() {
                Some(Component::Normal(_)) => {
                    components.pop();
                }
                // The root has no parent
                Some(Component::RootDir | Component::Prefix(_)) => {}
                _ => components.push(component),
            },
            _ => components.push(component),
        }
    }
    components.iter().collect()
}

/// Fixes spaces around `::` created by stringify!($function).
pub fn get_formated_function_path(function_path: impl Into<String>) -> String {
    let function_path = function_path.into();
    function_path.replace(" :: ", "::")
}

pub fn running_with_codspeed_runner() -> bool {
    std::env::var("CODSPEED_ENV").is_ok()
}

pub fn is_perf_enabled() -> bool {
    std::env::var("CODSPEED_PERF_ENABLED").is_ok()
}

/// Generate a statistically unique ID in a format resembling UUID v4.
pub fn generate_unique_id() -> String {
    // Generate random bytes for UUID v4
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");

    // Extract values from bytes
    let r1 = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let r2 = u16::from_be_bytes([bytes[4], bytes[5]]);
    let r3 = u16::from_be_bytes([bytes[6], bytes[7]]);
    let r4 = u16::from_be_bytes([bytes[8], bytes[9]]);
    let r5 = u32::from_be_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
    let r6 = u16::from_be_bytes([bytes[14], bytes[15]]);

    // Set version (4) and variant bits according to UUID v4 spec
    let r3_v4 = (r3 & 0x0fff) | 0x4000; // Version 4
    let r4_variant = (r4 & 0x3fff) | 0x8000; // Variant 10

    // Format as standard UUID: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    // where y is one of 8, 9, A, or B
    format!("{r1:08x}-{r2:04x}-{r3_v4:04x}-{r4_variant:04x}-{r5:08x}{r6:04x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_git_relative_path_found() {
        // Create a temp directory.
        let dir = tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir(git_dir).unwrap();
        let nested_dir = dir.path().join("folder").join("nested_folder");
        fs::create_dir_all(&nested_dir).unwrap();

        let relative_path = get_git_relative_path(&nested_dir);
        assert_eq!(relative_path, PathBuf::from("folder/nested_folder"));
    }

    #[test]
    fn test_get_git_relative_path_not_found() {
        let dir = tempdir().unwrap();
        let path_dir = dir.path().join("folder");
        fs::create_dir_all(&path_dir).unwrap();

        let relative_path = get_git_relative_path(&path_dir);
        assert_eq!(relative_path, path_dir.canonicalize().unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn test_get_git_relative_path_not_found_with_symlink() {
        let dir = tempdir().unwrap();
        let path_dir = dir.path().join("folder");
        fs::create_dir_all(&path_dir).unwrap();
        let symlink = dir.path().join("symlink");
        std::os::unix::fs::symlink(&path_dir, &symlink).unwrap();

        let relative_path = get_git_relative_path(&symlink);
        assert_eq!(relative_path, symlink.canonicalize().unwrap());
    }

    #[test]
    fn test_get_git_relative_path_resolves_dot_segments() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        let src_dir = dir.path().join("rust").join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("x.rs"), "").unwrap();

        // The file exists: the path is canonicalized
        let existing = dir.path().join("rust/benches/../src/./x.rs");
        assert_eq!(
            get_git_relative_path(existing),
            PathBuf::from("rust/src/x.rs")
        );
        // The file doesn't exist: the path is normalized
        let missing = dir.path().join("rust/benches/../src/./missing.rs");
        assert_eq!(
            get_git_relative_path(missing),
            PathBuf::from("rust/src/missing.rs")
        );
    }

    #[test]
    fn test_normalize_path_removes_dot_segments() {
        let cases = [
            ("crates/p/benches/../src/x.rs", "crates/p/src/x.rs"),
            ("crates/p/./benches/x.rs", "crates/p/benches/x.rs"),
            ("./benches/x.rs", "benches/x.rs"),
            ("benches/../../shared/x.rs", "../shared/x.rs"),
            ("../../x.rs", "../../x.rs"),
            ("benches/bench.rs", "benches/bench.rs"),
            ("a/.../b.rs", "a/.../b.rs"),
        ];
        for (file, expected) in cases {
            assert_eq!(
                normalize_path(Path::new(file)),
                Path::new(expected),
                "{file}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_normalize_path_keeps_absolute_paths_absolute() {
        assert_eq!(
            normalize_path(Path::new("/home/ci/ws/a/../b.rs")),
            Path::new("/home/ci/ws/b.rs")
        );
        assert_eq!(normalize_path(Path::new("/../x.rs")), Path::new("/x.rs"));
    }

    /// COD-2324: empty segments (no file/macro group when `criterion_group!` is bypassed)
    /// must not produce URIs like `::::my_group::my_bench`.
    #[test]
    fn test_build_uri_skips_empty_segments() {
        assert_eq!(
            build_uri(&["", "", "my_group::my_bench"]),
            "my_group::my_bench"
        );
        assert_eq!(
            build_uri(&["benches/custom_main.rs", "", "my_group::my_bench"]),
            "benches/custom_main.rs::my_group::my_bench"
        );
        assert_eq!(
            build_uri(&[
                "benches/bench.rs",
                "benches::bench_fn",
                "my_group::my_bench"
            ]),
            "benches/bench.rs::benches::bench_fn::my_group::my_bench"
        );
    }

    #[test]
    fn test_get_formated_function_path() {
        let input = "std :: vec :: Vec :: new";
        let expected_output = "std::vec::Vec::new".to_string();
        assert_eq!(get_formated_function_path(input), expected_output);
    }
}
