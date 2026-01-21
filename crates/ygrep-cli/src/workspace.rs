//! Workspace resolver with parent-directory index discovery
//!
//! This module provides functionality to discover ygrep indexes in parent
//! directories, enabling searches from subdirectories without explicit workspace
//! specification.

use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::xxh3_64;

/// Maximum depth for parent directory search
const MAX_PARENT_DEPTH: usize = 10;

/// Calculate workspace hash using xxh3_64 (same algorithm as core)
pub fn hash_workspace_path(path: &Path) -> String {
    let hash = xxh3_64(path.to_string_lossy().as_bytes());
    format!("{:016x}", hash)
}

/// Get the index directory for a given workspace hash
pub fn get_index_path_for_hash(data_dir: &Path, hash: &str) -> PathBuf {
    data_dir.join("indexes").join(hash)
}

/// Check if an index exists at the given path
pub fn index_exists(index_path: &Path) -> bool {
    index_path.join("workspace.json").exists()
}

/// Discover existing indexes in parent directories
///
/// Searches up to `MAX_PARENT_DEPTH` directories for existing ygrep indexes.
/// Returns a vector of (directory_path, is_indexed) pairs for directories
/// that have or are part of indexed workspaces.
///
/// The search stops early if:
/// - The filesystem root is reached
/// - MAX_PARENT_DEPTH is exceeded
/// - A directory with a workspace.json is found (this is the indexed workspace)
pub fn discover_parent_indexes(start_path: &Path, data_dir: Option<&Path>) -> Vec<(PathBuf, bool)> {
    let mut results = Vec::new();
    let mut current = match std::fs::canonicalize(start_path) {
        Ok(p) => p,
        Err(_) => return results,
    };

    // Get data directory (use default if not provided)
    let data_dir = match data_dir {
        Some(d) => d.to_path_buf(),
        None => default_data_dir(),
    };

    for _depth in 0..MAX_PARENT_DEPTH {
        // Calculate hash for this directory
        let hash = hash_workspace_path(&current);
        let index_path = get_index_path_for_hash(&data_dir, &hash);

        // Check if this directory has an index
        let is_indexed = index_exists(&index_path);
        results.push((current.clone(), is_indexed));

        // If we found an indexed workspace, we're done
        if is_indexed {
            break;
        }

        // Move to parent directory
        if let Some(parent) = current.parent() {
            if parent == current {
                // We've reached the root
                break;
            }
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    results
}

/// Find the nearest indexed parent directory
///
/// Returns the path to the nearest parent directory that has been indexed,
/// or None if no parent directory has an index.
pub fn find_nearest_indexed_parent(start_path: &Path, data_dir: Option<&Path>) -> Option<PathBuf> {
    for (path, is_indexed) in discover_parent_indexes(start_path, data_dir) {
        if is_indexed {
            return Some(path);
        }
    }
    None
}

/// Resolve the workspace path for a given starting path
///
/// If an explicit workspace is provided via -C, use that.
/// Otherwise, search parent directories for existing indexes.
///
/// Returns:
/// - Ok(Some(path)) if a workspace is found
/// - Ok(None) if no explicit workspace and no parent index found
/// - Err(e) if an error occurs
pub fn resolve_workspace(
    explicit_workspace: Option<&Path>,
    start_path: &Path,
    data_dir: Option<&Path>,
) -> Result<Option<PathBuf>, ResolveError> {
    // If explicit workspace is provided, use it
    if let Some(ws) = explicit_workspace {
        let canonical = std::fs::canonicalize(ws).map_err(|e| ResolveError::InvalidPath {
            path: ws.to_path_buf(),
            source: e,
        })?;

        // Verify the workspace is indexed
        let hash = hash_workspace_path(&canonical);
        let data_dir_ref = data_dir.unwrap_or_else(|| {
            // We need to store this somewhere so we can pass a reference
            // Use a static lazy initialization instead
            use std::sync::OnceLock;
            static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
            DATA_DIR.get_or_init(default_data_dir)
        });
        let index_path = get_index_path_for_hash(data_dir_ref, &hash);

        if !index_exists(&index_path) {
            return Err(ResolveError::NotIndexed {
                path: canonical.clone(),
            });
        }

        return Ok(Some(canonical));
    }

    // Search for parent index
    if let Some(indexed_parent) = find_nearest_indexed_parent(start_path, data_dir) {
        return Ok(Some(indexed_parent));
    }

    Ok(None)
}

/// Get the default data directory
fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("ygrep")
}

/// Errors that can occur when resolving a workspace
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum ResolveError {
    #[error("Invalid workspace path: {path}")]
    InvalidPath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Workspace is not indexed: {path}")]
    NotIndexed { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_hash_workspace_path() {
        // Test that hash is consistent and has correct format
        let path = PathBuf::from("/test/workspace");
        let hash = hash_workspace_path(&path);

        // Should be 16 hex characters
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Same path should produce same hash
        let hash2 = hash_workspace_path(&path);
        assert_eq!(hash, hash2);

        // Different path should produce different hash
        let path2 = PathBuf::from("/test/workspace2");
        let hash3 = hash_workspace_path(&path2);
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_discover_parent_indexes_empty() {
        let temp = tempdir().unwrap();
        let results = discover_parent_indexes(temp.path(), None);

        // Should have at least the starting directory
        assert!(!results.is_empty());
        assert_eq!(results[0].0, temp.path().canonicalize().unwrap());
        // Starting dir should not be indexed
        assert!(!results[0].1);
    }

    #[test]
    fn test_discover_parent_indexes_with_index() {
        let temp = tempdir().unwrap();
        let canonical = temp.path().canonicalize().unwrap();
        let data_dir = temp.path().join("data");

        // Create a fake index using canonical path
        let hash = hash_workspace_path(&canonical);
        let index_path = data_dir.join("indexes").join(&hash);
        std::fs::create_dir_all(&index_path).unwrap();
        std::fs::write(index_path.join("workspace.json"), "{}").unwrap();

        let results = discover_parent_indexes(temp.path(), Some(&data_dir));

        // First result should be indexed
        assert!(results[0].1);
    }

    #[test]
    fn test_find_nearest_indexed_parent() {
        let temp = tempdir().unwrap();
        let parent = temp.path().parent().unwrap().to_path_buf();
        let canonical_parent = parent.canonicalize().unwrap();

        // Create a fake index in the parent
        let hash = hash_workspace_path(&canonical_parent);
        let data_dir = temp.path().join("data");
        let index_path = data_dir.join("indexes").join(&hash);
        std::fs::create_dir_all(&index_path).unwrap();
        std::fs::write(index_path.join("workspace.json"), "{}").unwrap();

        let result = find_nearest_indexed_parent(temp.path(), Some(&data_dir));

        assert_eq!(result, Some(canonical_parent));
    }

    #[test]
    fn test_resolve_workspace_explicit() {
        let temp = tempdir().unwrap();
        let canonical = temp.path().canonicalize().unwrap();

        // Create a fake index using canonical path
        let hash = hash_workspace_path(&canonical);
        let index_path = temp.path().join("data").join("indexes").join(&hash);
        std::fs::create_dir_all(&index_path).unwrap();
        std::fs::write(index_path.join("workspace.json"), "{}").unwrap();

        let result = resolve_workspace(
            Some(&canonical),
            temp.path(),
            Some(&temp.path().join("data")),
        );

        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_resolve_workspace_explicit_not_indexed() {
        let temp = tempdir().unwrap();

        let result = resolve_workspace(Some(temp.path()), temp.path(), None);

        assert!(matches!(result, Err(ResolveError::NotIndexed { .. })));
    }

    #[test]
    fn test_resolve_workspace_parent_discovery() {
        let temp = tempdir().unwrap();
        let parent = temp.path().parent().unwrap().to_path_buf();
        let canonical_parent = parent.canonicalize().unwrap();

        // Create a fake index in the parent
        let hash = hash_workspace_path(&canonical_parent);
        let data_dir = temp.path().join("data");
        let index_path = data_dir.join("indexes").join(&hash);
        std::fs::create_dir_all(&index_path).unwrap();
        std::fs::write(index_path.join("workspace.json"), "{}").unwrap();

        let result = resolve_workspace(None, temp.path(), Some(&data_dir));

        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_max_depth_limit() {
        // Create a deep directory structure
        let temp = tempdir().unwrap();
        let mut path = temp.path().to_path_buf();

        // Create nested directories (more than MAX_PARENT_DEPTH)
        for i in 0..MAX_PARENT_DEPTH + 5 {
            path = path.join(format!("level_{}", i));
            std::fs::create_dir_all(&path).unwrap();
        }

        let results = discover_parent_indexes(&path, None);

        // Should not exceed max depth
        assert!(results.len() <= MAX_PARENT_DEPTH + 1);
    }
}
