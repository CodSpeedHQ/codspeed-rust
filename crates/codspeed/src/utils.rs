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
