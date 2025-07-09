//! Cross-platform path utilities for handling Windows and Unix paths

use std::path::{Path, PathBuf};

/// Normalize path for cross-platform compatibility
///
/// Converts backslashes to forward slashes for consistency.
/// Handles Windows absolute paths (C:/ etc) correctly.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    // Convert to string and replace backslashes
    let path_str = path.to_string_lossy().replace('\\', "/");

    // Handle Windows absolute paths (C:/ etc)
    if cfg!(windows) && path_str.len() > 2 && path_str.chars().nth(1) == Some(':') {
        // Already absolute Windows path
        return PathBuf::from(path_str);
    }

    PathBuf::from(path_str)
}

/// Get canonical path with fallback
///
/// Attempts to canonicalize the path, but falls back to normalization
/// if the path doesn't exist yet (useful for paths that will be created).
pub fn canonical_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref()
        .canonicalize()
        .unwrap_or_else(|_| normalize_path(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_unix() {
        let path = Path::new("/home/user/file.txt");
        let normalized = normalize_path(path);
        assert_eq!(normalized.to_string_lossy(), "/home/user/file.txt");
    }

    #[test]
    fn test_normalize_path_windows_backslashes() {
        let path = Path::new("C:\\Users\\test\\file.txt");
        let normalized = normalize_path(path);
        assert_eq!(normalized.to_string_lossy(), "C:/Users/test/file.txt");
    }

    #[test]
    fn test_normalize_path_mixed() {
        let path = Path::new("some/path\\with\\mixed/separators");
        let normalized = normalize_path(path);
        assert_eq!(
            normalized.to_string_lossy(),
            "some/path/with/mixed/separators"
        );
    }
}
