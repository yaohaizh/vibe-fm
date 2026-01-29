//! Integration tests for file operations
//!
//! These tests verify that file operations work correctly across
//! multiple operations and edge cases.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

// Helper to create a test file
fn create_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

// Helper to create a test directory
fn create_dir(dir: &Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::create_dir(&path).unwrap();
    path
}

/// Test copying a complex directory structure
#[test]
fn test_copy_complex_directory_structure() {
    let temp = TempDir::new().unwrap();

    // Create a complex directory structure
    // root/
    //   file1.txt
    //   subdir1/
    //     file2.txt
    //     subdir2/
    //       file3.txt
    //   subdir3/
    //     (empty)

    let root = create_dir(temp.path(), "root");
    create_file(&root, "file1.txt", "content1");

    let subdir1 = create_dir(&root, "subdir1");
    create_file(&subdir1, "file2.txt", "content2");

    let subdir2 = create_dir(&subdir1, "subdir2");
    create_file(&subdir2, "file3.txt", "content3");

    let _subdir3 = create_dir(&root, "subdir3"); // empty dir

    // Copy the entire structure
    let dest = temp.path().join("copy");
    copy_dir_all(&root, &dest).unwrap();

    // Verify structure
    assert!(dest.exists());
    assert!(dest.join("file1.txt").exists());
    assert!(dest.join("subdir1").exists());
    assert!(dest.join("subdir1").join("file2.txt").exists());
    assert!(dest.join("subdir1").join("subdir2").exists());
    assert!(dest
        .join("subdir1")
        .join("subdir2")
        .join("file3.txt")
        .exists());
    assert!(dest.join("subdir3").exists());

    // Verify content
    assert_eq!(
        fs::read_to_string(dest.join("file1.txt")).unwrap(),
        "content1"
    );
    assert_eq!(
        fs::read_to_string(dest.join("subdir1").join("file2.txt")).unwrap(),
        "content2"
    );
}

/// Test that operations preserve file content
#[test]
fn test_file_content_preservation() {
    let temp = TempDir::new().unwrap();

    // Create file with specific content including special characters
    let content = "Hello, World!\nLine 2\tTabbed\r\nWindows line\0Null byte";
    let source = create_file(temp.path(), "source.txt", content);

    // Copy
    let dest = temp.path().join("copy.txt");
    fs::copy(&source, &dest).unwrap();

    assert_eq!(fs::read(&source).unwrap(), fs::read(&dest).unwrap());
}

/// Test moving files between directories
#[test]
fn test_move_between_directories() {
    let temp = TempDir::new().unwrap();

    let dir1 = create_dir(temp.path(), "dir1");
    let dir2 = create_dir(temp.path(), "dir2");

    let file = create_file(&dir1, "moveme.txt", "content");
    let dest = dir2.join("moveme.txt");

    fs::rename(&file, &dest).unwrap();

    assert!(!file.exists());
    assert!(dest.exists());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "content");
}

/// Test deleting non-empty directories
#[test]
fn test_delete_non_empty_directory() {
    let temp = TempDir::new().unwrap();

    let dir = create_dir(temp.path(), "to_delete");
    create_file(&dir, "file1.txt", "content1");
    create_file(&dir, "file2.txt", "content2");
    let subdir = create_dir(&dir, "subdir");
    create_file(&subdir, "file3.txt", "content3");

    fs::remove_dir_all(&dir).unwrap();

    assert!(!dir.exists());
}

/// Test creating files in nested non-existent directories
#[test]
fn test_create_nested_directories() {
    let temp = TempDir::new().unwrap();

    let nested = temp.path().join("a").join("b").join("c");
    fs::create_dir_all(&nested).unwrap();

    assert!(nested.exists());
    assert!(nested.is_dir());
}

/// Test handling of read-only files (platform dependent)
#[test]
#[cfg(unix)]
fn test_read_only_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let file = create_file(temp.path(), "readonly.txt", "content");

    // Make file read-only
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&file, perms).unwrap();

    // Verify it's read-only
    let metadata = fs::metadata(&file).unwrap();
    assert!(metadata.permissions().readonly());

    // Cleanup: make writable again so temp dir can be deleted
    let mut perms = metadata.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&file, perms).unwrap();
}

/// Test handling of special filenames
#[test]
fn test_special_filenames() {
    let temp = TempDir::new().unwrap();

    // Files starting with dot (hidden on Unix)
    let hidden = create_file(temp.path(), ".hidden", "content");
    assert!(hidden.exists());

    // Files with spaces
    let spaced = create_file(temp.path(), "file with spaces.txt", "content");
    assert!(spaced.exists());

    // Files with unicode
    let unicode = create_file(temp.path(), "файл.txt", "content");
    assert!(unicode.exists());
}

/// Test large file operations
#[test]
fn test_large_file_handling() {
    let temp = TempDir::new().unwrap();

    // Create a 1MB file
    let source = temp.path().join("large.bin");
    let data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
    fs::write(&source, &data).unwrap();

    // Copy it
    let dest = temp.path().join("large_copy.bin");
    fs::copy(&source, &dest).unwrap();

    // Verify
    assert_eq!(fs::metadata(&dest).unwrap().len(), 1024 * 1024);
    assert_eq!(fs::read(&source).unwrap(), fs::read(&dest).unwrap());
}

/// Test concurrent file access (basic)
#[test]
fn test_multiple_file_operations() {
    let temp = TempDir::new().unwrap();

    // Create multiple files
    for i in 0..10 {
        create_file(
            temp.path(),
            &format!("file{}.txt", i),
            &format!("content{}", i),
        );
    }

    // Count files
    let count = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count();

    assert_eq!(count, 10);

    // Delete half
    for i in 0..5 {
        fs::remove_file(temp.path().join(format!("file{}.txt", i))).unwrap();
    }

    // Verify remaining
    let count = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count();

    assert_eq!(count, 5);
}

// Helper function for recursive directory copy
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
