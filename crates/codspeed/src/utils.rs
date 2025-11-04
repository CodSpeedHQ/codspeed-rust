use std::io;
use std::path::{Path, PathBuf};

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

pub fn get_git_relative_path<P>(abs_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    if let Ok(canonicalized_abs_path) = abs_path.as_ref().canonicalize() {
        // `repo_path` is still canonicalized as it is a subpath of `canonicalized_abs_path`
        if let Ok(repo_path) = get_parent_git_repo_path(&canonicalized_abs_path) {
            canonicalized_abs_path
                .strip_prefix(repo_path)
                .expect("Repository path is malformed.")
                .to_path_buf()
        } else {
            canonicalized_abs_path
        }
    } else {
        abs_path.as_ref().to_path_buf()
    }
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

pub fn show_details() -> bool {
    std::env::var("CODSPEED_SHOW_DETAILS").is_ok()
}

/// Format a duration value (as f64 nanoseconds) into a human-readable string with appropriate units
pub fn format_duration_nanos_f64(value: f64) -> String {
    if value <= 0.0 {
        return "<1 ns".to_string();
    }

    if value < 1_000.0 {
        format!("{value:.0} ns")
    } else if value < 1_000_000.0 {
        format!("{:.1} us", value / 1_000.0)
    } else if value < 1_000_000_000.0 {
        format!("{:.1} ms", value / 1_000_000.0)
    } else {
        format!("{:.2} s", value / 1_000_000_000.0)
    }
}

/// Format a duration value into a human-readable string with appropriate units
pub fn format_duration_nanos(nanos: u128) -> String {
    format_duration_nanos_f64(nanos as f64)
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
    fn test_get_formated_function_path() {
        let input = "std :: vec :: Vec :: new";
        let expected_output = "std::vec::Vec::new".to_string();
        assert_eq!(get_formated_function_path(input), expected_output);
    }

    #[test]
    fn test_format_duration_nanos_handles_zero() {
        assert_eq!(format_duration_nanos(0), "<1 ns");
    }

    #[test]
    fn test_format_duration_nanos_scales_to_microseconds() {
        assert_eq!(format_duration_nanos(1_500), "1.5 us");
    }

    #[test]
    fn test_format_duration_nanos_scales_to_milliseconds() {
        assert_eq!(format_duration_nanos(2_345_000), "2.3 ms");
    }

    #[test]
    fn test_format_duration_nanos_scales_to_seconds() {
        assert_eq!(format_duration_nanos(2_500_000_000), "2.50 s");
    }

    #[test]
    fn test_format_duration_nanos_f64_handles_zero() {
        assert_eq!(format_duration_nanos_f64(0.0), "<1 ns");
    }

    #[test]
    fn test_format_duration_nanos_f64_scales_to_microseconds() {
        assert_eq!(format_duration_nanos_f64(1_500.0), "1.5 us");
    }

    #[test]
    fn test_format_duration_nanos_f64_scales_to_milliseconds() {
        assert_eq!(format_duration_nanos_f64(2_345_000.0), "2.3 ms");
    }

    #[test]
    fn test_format_duration_nanos_f64_scales_to_seconds() {
        assert_eq!(format_duration_nanos_f64(2_500_000_000.0), "2.50 s");
    }

    // Additional format_duration_nanos_f64 tests (moved from walltime_results.rs)
    #[test]
    fn test_format_duration_nanos_f64_formats_small_values() {
        assert_eq!(format_duration_nanos_f64(420.0), "420 ns");
    }

    #[test]
    fn test_format_duration_nanos_f64_formats_microseconds() {
        assert_eq!(format_duration_nanos_f64(12_500.0), "12.5 us");
    }

    #[test]
    fn test_format_duration_nanos_f64_formats_milliseconds() {
        assert_eq!(format_duration_nanos_f64(3_250_000.0), "3.2 ms");
    }

    #[test]
    fn test_format_duration_nanos_f64_formats_seconds() {
        assert_eq!(format_duration_nanos_f64(1_750_000_000.0), "1.75 s");
    }
}
