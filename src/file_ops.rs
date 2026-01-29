//! File operations module
//!
//! This module contains file system operations that can be tested independently
//! from the UI components.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Result type for file operations
pub type FileOpResult<T> = Result<T, FileOpError>;

/// Errors that can occur during file operations
#[derive(Debug)]
pub enum FileOpError {
    IoError(io::Error),
    SourceNotFound(PathBuf),
    DestinationExists(PathBuf),
    InvalidPath(String),
    PermissionDenied(PathBuf),
}

impl From<io::Error> for FileOpError {
    fn from(error: io::Error) -> Self {
        FileOpError::IoError(error)
    }
}

impl std::fmt::Display for FileOpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOpError::IoError(e) => write!(f, "IO error: {}", e),
            FileOpError::SourceNotFound(p) => write!(f, "Source not found: {:?}", p),
            FileOpError::DestinationExists(p) => write!(f, "Destination already exists: {:?}", p),
            FileOpError::InvalidPath(s) => write!(f, "Invalid path: {}", s),
            FileOpError::PermissionDenied(p) => write!(f, "Permission denied: {:?}", p),
        }
    }
}

impl std::error::Error for FileOpError {}

/// Copies a file from source to destination
pub fn copy_file(source: &Path, dest: &Path) -> FileOpResult<u64> {
    if !source.exists() {
        return Err(FileOpError::SourceNotFound(source.to_path_buf()));
    }

    let bytes = fs::copy(source, dest)?;
    Ok(bytes)
}

/// Copies a directory recursively from source to destination
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> FileOpResult<()> {
    if !src.exists() {
        return Err(FileOpError::SourceNotFound(src.to_path_buf()));
    }

    if !src.is_dir() {
        return Err(FileOpError::InvalidPath(format!(
            "{:?} is not a directory",
            src
        )));
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Moves a file or directory from source to destination
pub fn move_item(source: &Path, dest: &Path) -> FileOpResult<()> {
    if !source.exists() {
        return Err(FileOpError::SourceNotFound(source.to_path_buf()));
    }

    fs::rename(source, dest)?;
    Ok(())
}

/// Deletes a file
pub fn delete_file(path: &Path) -> FileOpResult<()> {
    if !path.exists() {
        return Err(FileOpError::SourceNotFound(path.to_path_buf()));
    }

    fs::remove_file(path)?;
    Ok(())
}

/// Deletes a directory and all its contents
pub fn delete_dir(path: &Path) -> FileOpResult<()> {
    if !path.exists() {
        return Err(FileOpError::SourceNotFound(path.to_path_buf()));
    }

    fs::remove_dir_all(path)?;
    Ok(())
}

/// Deletes a file or directory
pub fn delete_item(path: &Path) -> FileOpResult<()> {
    if path.is_dir() {
        delete_dir(path)
    } else {
        delete_file(path)
    }
}

/// Creates a new directory
pub fn create_directory(path: &Path) -> FileOpResult<()> {
    if path.exists() {
        return Err(FileOpError::DestinationExists(path.to_path_buf()));
    }

    fs::create_dir(path)?;
    Ok(())
}

/// Creates a new empty file
pub fn create_file(path: &Path) -> FileOpResult<()> {
    if path.exists() {
        return Err(FileOpError::DestinationExists(path.to_path_buf()));
    }

    fs::File::create(path)?;
    Ok(())
}

/// Renames a file or directory
pub fn rename_item(source: &Path, new_name: &str) -> FileOpResult<PathBuf> {
    if !source.exists() {
        return Err(FileOpError::SourceNotFound(source.to_path_buf()));
    }

    let parent = source
        .parent()
        .ok_or_else(|| FileOpError::InvalidPath("Cannot get parent directory".to_string()))?;

    let new_path = parent.join(new_name);

    if new_path.exists() {
        return Err(FileOpError::DestinationExists(new_path));
    }

    fs::rename(source, &new_path)?;
    Ok(new_path)
}

/// Gets the size of a file or directory (recursive for directories)
pub fn get_size(path: &Path) -> FileOpResult<u64> {
    if !path.exists() {
        return Err(FileOpError::SourceNotFound(path.to_path_buf()));
    }

    if path.is_file() {
        let metadata = fs::metadata(path)?;
        return Ok(metadata.len());
    }

    let mut total_size = 0u64;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            total_size += fs::metadata(&entry_path)?.len();
        } else if entry_path.is_dir() {
            total_size += get_size(&entry_path)?;
        }
    }

    Ok(total_size)
}

/// Counts files and directories in a path
pub fn count_items(path: &Path) -> FileOpResult<(usize, usize)> {
    if !path.exists() {
        return Err(FileOpError::SourceNotFound(path.to_path_buf()));
    }

    if path.is_file() {
        return Ok((1, 0));
    }

    let mut file_count = 0;
    let mut dir_count = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            file_count += 1;
        } else if entry_path.is_dir() {
            dir_count += 1;
        }
    }

    Ok((file_count, dir_count))
}

/// Checks if a path is writable
pub fn is_writable(path: &Path) -> bool {
    if path.exists() {
        // Try to get metadata with write access
        fs::metadata(path)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false)
    } else {
        // Check if parent directory is writable
        path.parent().map(|p| is_writable(p)).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // ==================== Helper Functions ====================

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    fn create_test_dir(dir: &Path, name: &str) -> PathBuf {
        let dir_path = dir.join(name);
        fs::create_dir(&dir_path).unwrap();
        dir_path
    }

    // ==================== Copy File Tests ====================

    #[test]
    fn test_copy_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_file(temp_dir.path(), "source.txt", "Hello, World!");
        let dest = temp_dir.path().join("dest.txt");

        let result = copy_file(&source, &dest);

        assert!(result.is_ok());
        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "Hello, World!");
    }

    #[test]
    fn test_copy_file_source_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("nonexistent.txt");
        let dest = temp_dir.path().join("dest.txt");

        let result = copy_file(&source, &dest);

        assert!(matches!(result, Err(FileOpError::SourceNotFound(_))));
    }

    #[test]
    fn test_copy_file_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_file(temp_dir.path(), "source.txt", "New Content");
        let dest = create_test_file(temp_dir.path(), "dest.txt", "Old Content");

        let result = copy_file(&source, &dest);

        assert!(result.is_ok());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "New Content");
    }

    // ==================== Copy Directory Tests ====================

    #[test]
    fn test_copy_dir_recursive_success() {
        let temp_dir = TempDir::new().unwrap();

        // Create source directory with files
        let source_dir = create_test_dir(temp_dir.path(), "source");
        create_test_file(&source_dir, "file1.txt", "Content 1");
        create_test_file(&source_dir, "file2.txt", "Content 2");

        // Create subdirectory
        let sub_dir = create_test_dir(&source_dir, "subdir");
        create_test_file(&sub_dir, "file3.txt", "Content 3");

        let dest_dir = temp_dir.path().join("dest");

        let result = copy_dir_recursive(&source_dir, &dest_dir);

        assert!(result.is_ok());
        assert!(dest_dir.exists());
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("file2.txt").exists());
        assert!(dest_dir.join("subdir").exists());
        assert!(dest_dir.join("subdir").join("file3.txt").exists());
    }

    #[test]
    fn test_copy_dir_recursive_source_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("nonexistent");
        let dest = temp_dir.path().join("dest");

        let result = copy_dir_recursive(&source, &dest);

        assert!(matches!(result, Err(FileOpError::SourceNotFound(_))));
    }

    #[test]
    fn test_copy_dir_recursive_source_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_file(temp_dir.path(), "file.txt", "content");
        let dest = temp_dir.path().join("dest");

        let result = copy_dir_recursive(&source, &dest);

        assert!(matches!(result, Err(FileOpError::InvalidPath(_))));
    }

    // ==================== Move Item Tests ====================

    #[test]
    fn test_move_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_file(temp_dir.path(), "source.txt", "Content");
        let dest = temp_dir.path().join("moved.txt");

        let result = move_item(&source, &dest);

        assert!(result.is_ok());
        assert!(!source.exists());
        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "Content");
    }

    #[test]
    fn test_move_directory_success() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_dir(temp_dir.path(), "source_dir");
        create_test_file(&source, "file.txt", "Content");
        let dest = temp_dir.path().join("moved_dir");

        let result = move_item(&source, &dest);

        assert!(result.is_ok());
        assert!(!source.exists());
        assert!(dest.exists());
        assert!(dest.join("file.txt").exists());
    }

    #[test]
    fn test_move_item_source_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("nonexistent");
        let dest = temp_dir.path().join("dest");

        let result = move_item(&source, &dest);

        assert!(matches!(result, Err(FileOpError::SourceNotFound(_))));
    }

    // ==================== Delete Tests ====================

    #[test]
    fn test_delete_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file(temp_dir.path(), "to_delete.txt", "Content");

        let result = delete_file(&file);

        assert!(result.is_ok());
        assert!(!file.exists());
    }

    #[test]
    fn test_delete_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("nonexistent.txt");

        let result = delete_file(&file);

        assert!(matches!(result, Err(FileOpError::SourceNotFound(_))));
    }

    #[test]
    fn test_delete_dir_success() {
        let temp_dir = TempDir::new().unwrap();
        let dir = create_test_dir(temp_dir.path(), "to_delete");
        create_test_file(&dir, "file.txt", "Content");

        let result = delete_dir(&dir);

        assert!(result.is_ok());
        assert!(!dir.exists());
    }

    #[test]
    fn test_delete_item_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file(temp_dir.path(), "file.txt", "Content");

        let result = delete_item(&file);

        assert!(result.is_ok());
        assert!(!file.exists());
    }

    #[test]
    fn test_delete_item_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir = create_test_dir(temp_dir.path(), "dir");

        let result = delete_item(&dir);

        assert!(result.is_ok());
        assert!(!dir.exists());
    }

    // ==================== Create Tests ====================

    #[test]
    fn test_create_directory_success() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_dir");

        let result = create_directory(&new_dir);

        assert!(result.is_ok());
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_create_directory_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let existing_dir = create_test_dir(temp_dir.path(), "existing");

        let result = create_directory(&existing_dir);

        assert!(matches!(result, Err(FileOpError::DestinationExists(_))));
    }

    #[test]
    fn test_create_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let new_file = temp_dir.path().join("new_file.txt");

        let result = create_file(&new_file);

        assert!(result.is_ok());
        assert!(new_file.exists());
        assert!(new_file.is_file());
    }

    #[test]
    fn test_create_file_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = create_test_file(temp_dir.path(), "existing.txt", "Content");

        let result = create_file(&existing_file);

        assert!(matches!(result, Err(FileOpError::DestinationExists(_))));
    }

    // ==================== Rename Tests ====================

    #[test]
    fn test_rename_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file(temp_dir.path(), "old_name.txt", "Content");

        let result = rename_item(&file, "new_name.txt");

        assert!(result.is_ok());
        let new_path = result.unwrap();
        assert!(!file.exists());
        assert!(new_path.exists());
        assert_eq!(
            new_path.file_name().unwrap().to_str().unwrap(),
            "new_name.txt"
        );
    }

    #[test]
    fn test_rename_directory_success() {
        let temp_dir = TempDir::new().unwrap();
        let dir = create_test_dir(temp_dir.path(), "old_dir");

        let result = rename_item(&dir, "new_dir");

        assert!(result.is_ok());
        let new_path = result.unwrap();
        assert!(!dir.exists());
        assert!(new_path.exists());
        assert!(new_path.is_dir());
    }

    #[test]
    fn test_rename_source_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("nonexistent.txt");

        let result = rename_item(&file, "new_name.txt");

        assert!(matches!(result, Err(FileOpError::SourceNotFound(_))));
    }

    #[test]
    fn test_rename_destination_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = create_test_file(temp_dir.path(), "file1.txt", "Content 1");
        create_test_file(temp_dir.path(), "file2.txt", "Content 2");

        let result = rename_item(&file1, "file2.txt");

        assert!(matches!(result, Err(FileOpError::DestinationExists(_))));
    }

    // ==================== Size Tests ====================

    #[test]
    fn test_get_size_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file(temp_dir.path(), "file.txt", "Hello, World!"); // 13 bytes

        let result = get_size(&file);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 13);
    }

    #[test]
    fn test_get_size_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir = create_test_dir(temp_dir.path(), "dir");
        create_test_file(&dir, "file1.txt", "12345"); // 5 bytes
        create_test_file(&dir, "file2.txt", "67890"); // 5 bytes

        let result = get_size(&dir);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_get_size_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir = create_test_dir(temp_dir.path(), "dir");
        create_test_file(&dir, "file1.txt", "12345"); // 5 bytes
        let sub_dir = create_test_dir(&dir, "subdir");
        create_test_file(&sub_dir, "file2.txt", "67890"); // 5 bytes

        let result = get_size(&dir);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_get_size_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("nonexistent.txt");

        let result = get_size(&file);

        assert!(matches!(result, Err(FileOpError::SourceNotFound(_))));
    }

    // ==================== Count Items Tests ====================

    #[test]
    fn test_count_items_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir = create_test_dir(temp_dir.path(), "dir");
        create_test_file(&dir, "file1.txt", "Content");
        create_test_file(&dir, "file2.txt", "Content");
        create_test_dir(&dir, "subdir1");
        create_test_dir(&dir, "subdir2");

        let result = count_items(&dir);

        assert!(result.is_ok());
        let (files, dirs) = result.unwrap();
        assert_eq!(files, 2);
        assert_eq!(dirs, 2);
    }

    #[test]
    fn test_count_items_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file(temp_dir.path(), "file.txt", "Content");

        let result = count_items(&file);

        assert!(result.is_ok());
        let (files, dirs) = result.unwrap();
        assert_eq!(files, 1);
        assert_eq!(dirs, 0);
    }

    #[test]
    fn test_count_items_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir = create_test_dir(temp_dir.path(), "empty_dir");

        let result = count_items(&dir);

        assert!(result.is_ok());
        let (files, dirs) = result.unwrap();
        assert_eq!(files, 0);
        assert_eq!(dirs, 0);
    }

    // ==================== Writable Tests ====================

    #[test]
    fn test_is_writable_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file(temp_dir.path(), "file.txt", "Content");

        // Temp files should be writable
        assert!(is_writable(&file));
    }

    #[test]
    fn test_is_writable_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir = create_test_dir(temp_dir.path(), "dir");

        assert!(is_writable(&dir));
    }

    #[test]
    fn test_is_writable_nonexistent_in_writable_parent() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent.txt");

        // Parent (temp_dir) should be writable
        assert!(is_writable(&nonexistent));
    }
}
