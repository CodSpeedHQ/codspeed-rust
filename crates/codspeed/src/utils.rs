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
    let abs_path = abs_path.as_ref();
    match abs_path
        .canonicalize()
        .and_then(|p| get_parent_git_repo_path(&p))
    {
        Ok(repo_path) => abs_path.strip_prefix(repo_path).unwrap().to_path_buf(),
        Err(_) => abs_path.to_path_buf(),
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
    fn test_get_formated_function_path() {
        let input = "std :: vec :: Vec :: new";
        let expected_output = "std::vec::Vec::new".to_string();
        assert_eq!(get_formated_function_path(input), expected_output);
    }
}
