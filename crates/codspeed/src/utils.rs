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

/// Generate a statistically unique ID in a format resembling UUID v4.
pub fn generate_unique_id() -> String {
    // Generate random bytes for UUID v4
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");

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
    fn test_get_formated_function_path() {
        let input = "std :: vec :: Vec :: new";
        let expected_output = "std::vec::Vec::new".to_string();
        assert_eq!(get_formated_function_path(input), expected_output);
    }
}
