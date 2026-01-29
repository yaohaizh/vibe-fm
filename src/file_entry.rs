use chrono::{DateTime, Local};
use gpui_component::IconName;
use humansize::{format_size, BINARY, DECIMAL};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::settings::{DateFormat, SizeFormat};

#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
    Directory,
    File,
    Symlink,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub file_type: FileType,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub is_hidden: bool,
    pub extension: Option<String>,
    pub is_selected: bool,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_symlink() {
            FileType::Symlink
        } else if metadata.is_file() {
            FileType::File
        } else {
            FileType::Unknown
        };

        let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase());

        let is_hidden = name.starts_with('.');

        Ok(Self {
            name,
            path,
            file_type,
            size: metadata.len(),
            modified: metadata.modified().ok(),
            is_hidden,
            extension,
            is_selected: false,
        })
    }

    pub fn parent_entry(path: PathBuf) -> Self {
        Self {
            name: "..".to_string(),
            path,
            file_type: FileType::Directory,
            size: 0,
            modified: None,
            is_hidden: false,
            extension: None,
            is_selected: false,
        }
    }

    pub fn formatted_size(&self) -> String {
        match self.file_type {
            FileType::Directory => "<DIR>".to_string(),
            _ => format_size(self.size, BINARY),
        }
    }

    /// Format size with the specified format setting
    pub fn formatted_size_with_format(&self, size_format: SizeFormat) -> String {
        match self.file_type {
            FileType::Directory => "<DIR>".to_string(),
            _ => match size_format {
                SizeFormat::Binary => format_size(self.size, BINARY),
                SizeFormat::Decimal => format_size(self.size, DECIMAL),
            },
        }
    }

    pub fn formatted_date(&self) -> String {
        self.modified
            .map(|t| {
                let datetime: DateTime<Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_default()
    }

    /// Format date with the specified format setting
    pub fn formatted_date_with_format(&self, date_format: DateFormat) -> String {
        self.modified
            .map(|t| {
                let datetime: DateTime<Local> = t.into();
                match date_format {
                    DateFormat::Relative => Self::format_relative_time(t),
                    _ => datetime.format(date_format.format_string()).to_string(),
                }
            })
            .unwrap_or_default()
    }

    /// Format time as relative (e.g., "2 hours ago", "Yesterday")
    fn format_relative_time(time: SystemTime) -> String {
        let now = SystemTime::now();
        let duration = match now.duration_since(time) {
            Ok(d) => d,
            Err(_) => return "In the future".to_string(),
        };

        let secs = duration.as_secs();
        let mins = secs / 60;
        let hours = mins / 60;
        let days = hours / 24;

        if secs < 60 {
            "Just now".to_string()
        } else if mins < 60 {
            format!("{} min ago", mins)
        } else if hours < 24 {
            format!("{} hours ago", hours)
        } else if days == 1 {
            "Yesterday".to_string()
        } else if days < 7 {
            format!("{} days ago", days)
        } else if days < 30 {
            format!("{} weeks ago", days / 7)
        } else if days < 365 {
            format!("{} months ago", days / 30)
        } else {
            format!("{} years ago", days / 365)
        }
    }

    /// Get display name, optionally hiding the extension
    pub fn display_name(&self, show_extension: bool) -> String {
        if show_extension || self.is_directory() || self.extension.is_none() {
            self.name.clone()
        } else {
            // Remove extension from name
            if let Some(ext) = &self.extension {
                let suffix = format!(".{}", ext);
                if self.name.to_lowercase().ends_with(&suffix) {
                    self.name[..self.name.len() - suffix.len()].to_string()
                } else {
                    self.name.clone()
                }
            } else {
                self.name.clone()
            }
        }
    }

    pub fn icon_name(&self) -> IconName {
        match self.file_type {
            FileType::Directory => IconName::Folder,
            FileType::Symlink => IconName::ExternalLink,
            FileType::File => self.get_file_icon(),
            FileType::Unknown => IconName::File,
        }
    }

    fn get_file_icon(&self) -> IconName {
        match self.extension.as_deref() {
            // Images
            Some(
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "raw",
            ) => IconName::GalleryVerticalEnd,
            // Documents/Text files
            Some("txt" | "md" | "log" | "csv" | "pdf" | "doc" | "docx" | "rtf" | "odt") => {
                IconName::BookOpen
            }
            // Code files
            Some(
                "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "html" | "css" | "scss" | "json"
                | "yaml" | "yml" | "toml" | "xml" | "c" | "cpp" | "h" | "hpp" | "java" | "go"
                | "rb" | "php" | "swift" | "kt" | "vue" | "svelte",
            ) => IconName::File,
            // Shell/Scripts
            Some("sh" | "bash" | "zsh" | "ps1" | "bat" | "cmd") => IconName::SquareTerminal,
            // Executables
            Some("exe" | "msi" | "app" | "bin") => IconName::SquareTerminal,
            // Web
            Some("htm" | "xhtml" | "url" | "link") => IconName::Globe,
            // Config/Settings
            Some("ini" | "conf" | "config" | "env" | "properties") => IconName::Settings,
            // Archives
            Some("zip" | "tar" | "gz" | "rar" | "7z" | "bz2" | "xz" | "iso") => IconName::Inbox,
            // Audio/Video (using Star as placeholder)
            Some(
                "mp3" | "wav" | "ogg" | "flac" | "aac" | "wma" | "m4a" | "mp4" | "avi" | "mkv"
                | "mov" | "wmv" | "flv" | "webm",
            ) => IconName::Star,
            // Git
            Some("git" | "gitignore" | "gitmodules") => IconName::GitHub,
            // Default
            _ => IconName::File,
        }
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortColumn {
    Name,
    Size,
    Modified,
    Extension,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

pub fn sort_entries(entries: &mut [FileEntry], column: &SortColumn, order: &SortOrder) {
    entries.sort_by(|a, b| {
        // Parent directory ".." always first
        if a.name == ".." {
            return std::cmp::Ordering::Less;
        }
        if b.name == ".." {
            return std::cmp::Ordering::Greater;
        }

        // Directories always before files
        match (&a.file_type, &b.file_type) {
            (FileType::Directory, FileType::Directory) => {}
            (FileType::Directory, _) => return std::cmp::Ordering::Less,
            (_, FileType::Directory) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        let cmp = match column {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.size.cmp(&b.size),
            SortColumn::Modified => a.modified.cmp(&b.modified),
            SortColumn::Extension => a.extension.cmp(&b.extension),
        };

        match order {
            SortOrder::Ascending => cmp,
            SortOrder::Descending => cmp.reverse(),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create a test FileEntry without hitting the filesystem
    fn create_test_entry(name: &str, file_type: FileType, size: u64) -> FileEntry {
        FileEntry {
            name: name.to_string(),
            path: PathBuf::from(format!("/test/{}", name)),
            file_type,
            size,
            modified: None,
            is_hidden: name.starts_with('.'),
            extension: PathBuf::from(name)
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase()),
            is_selected: false,
        }
    }

    // ==================== FileType Tests ====================

    #[test]
    fn test_file_type_equality() {
        assert_eq!(FileType::Directory, FileType::Directory);
        assert_eq!(FileType::File, FileType::File);
        assert_eq!(FileType::Symlink, FileType::Symlink);
        assert_eq!(FileType::Unknown, FileType::Unknown);
        assert_ne!(FileType::Directory, FileType::File);
    }

    // ==================== FileEntry Tests ====================

    #[test]
    fn test_file_entry_new_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();

        let entry = FileEntry::new(file_path.clone()).unwrap();

        assert_eq!(entry.name, "test_file.txt");
        assert_eq!(entry.file_type, FileType::File);
        assert_eq!(entry.size, 13); // "Hello, World!" is 13 bytes
        assert_eq!(entry.extension, Some("txt".to_string()));
        assert!(!entry.is_hidden);
        assert!(!entry.is_selected);
    }

    #[test]
    fn test_file_entry_new_with_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path).unwrap();

        let entry = FileEntry::new(dir_path.clone()).unwrap();

        assert_eq!(entry.name, "test_dir");
        assert_eq!(entry.file_type, FileType::Directory);
        assert!(entry.is_directory());
        assert_eq!(entry.extension, None);
    }

    #[test]
    fn test_file_entry_hidden_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".hidden_file");
        File::create(&file_path).unwrap();

        let entry = FileEntry::new(file_path).unwrap();

        assert!(entry.is_hidden);
        assert_eq!(entry.name, ".hidden_file");
    }

    #[test]
    fn test_file_entry_parent_entry() {
        let parent_path = PathBuf::from("/home/user");
        let entry = FileEntry::parent_entry(parent_path.clone());

        assert_eq!(entry.name, "..");
        assert_eq!(entry.path, parent_path);
        assert_eq!(entry.file_type, FileType::Directory);
        assert_eq!(entry.size, 0);
        assert!(entry.modified.is_none());
        assert!(!entry.is_hidden);
    }

    #[test]
    fn test_file_entry_is_directory() {
        let dir_entry = create_test_entry("folder", FileType::Directory, 0);
        let file_entry = create_test_entry("file.txt", FileType::File, 100);

        assert!(dir_entry.is_directory());
        assert!(!file_entry.is_directory());
    }

    // ==================== Formatted Size Tests ====================

    #[test]
    fn test_formatted_size_directory() {
        let entry = create_test_entry("folder", FileType::Directory, 0);
        assert_eq!(entry.formatted_size(), "<DIR>");
    }

    #[test]
    fn test_formatted_size_file() {
        let entry = create_test_entry("file.txt", FileType::File, 1024);
        // humansize uses binary format, 1024 bytes = 1 KiB
        assert!(entry.formatted_size().contains("Ki") || entry.formatted_size().contains("1024"));
    }

    #[test]
    fn test_formatted_size_zero() {
        let entry = create_test_entry("empty.txt", FileType::File, 0);
        let size = entry.formatted_size();
        assert!(size.contains("0") || size.contains("B"));
    }

    // ==================== Extension Detection Tests ====================

    #[test]
    fn test_extension_detection() {
        let entry = create_test_entry("document.pdf", FileType::File, 100);
        assert_eq!(entry.extension, Some("pdf".to_string()));
    }

    #[test]
    fn test_extension_uppercase() {
        // Extension should be lowercased
        let entry = FileEntry {
            name: "IMAGE.PNG".to_string(),
            path: PathBuf::from("/test/IMAGE.PNG"),
            file_type: FileType::File,
            size: 100,
            modified: None,
            is_hidden: false,
            extension: Some("png".to_string()), // Should be lowercase
            is_selected: false,
        };
        assert_eq!(entry.extension, Some("png".to_string()));
    }

    #[test]
    fn test_no_extension() {
        let entry = create_test_entry("README", FileType::File, 100);
        assert_eq!(entry.extension, None);
    }

    // ==================== Sorting Tests ====================

    #[test]
    fn test_sort_by_name_ascending() {
        let mut entries = vec![
            create_test_entry("zebra.txt", FileType::File, 100),
            create_test_entry("apple.txt", FileType::File, 100),
            create_test_entry("mango.txt", FileType::File, 100),
        ];

        sort_entries(&mut entries, &SortColumn::Name, &SortOrder::Ascending);

        assert_eq!(entries[0].name, "apple.txt");
        assert_eq!(entries[1].name, "mango.txt");
        assert_eq!(entries[2].name, "zebra.txt");
    }

    #[test]
    fn test_sort_by_name_descending() {
        let mut entries = vec![
            create_test_entry("apple.txt", FileType::File, 100),
            create_test_entry("zebra.txt", FileType::File, 100),
            create_test_entry("mango.txt", FileType::File, 100),
        ];

        sort_entries(&mut entries, &SortColumn::Name, &SortOrder::Descending);

        assert_eq!(entries[0].name, "zebra.txt");
        assert_eq!(entries[1].name, "mango.txt");
        assert_eq!(entries[2].name, "apple.txt");
    }

    #[test]
    fn test_sort_by_name_case_insensitive() {
        let mut entries = vec![
            create_test_entry("Zebra.txt", FileType::File, 100),
            create_test_entry("apple.txt", FileType::File, 100),
            create_test_entry("BANANA.txt", FileType::File, 100),
        ];

        sort_entries(&mut entries, &SortColumn::Name, &SortOrder::Ascending);

        assert_eq!(entries[0].name, "apple.txt");
        assert_eq!(entries[1].name, "BANANA.txt");
        assert_eq!(entries[2].name, "Zebra.txt");
    }

    #[test]
    fn test_sort_by_size_ascending() {
        let mut entries = vec![
            create_test_entry("large.txt", FileType::File, 1000),
            create_test_entry("small.txt", FileType::File, 10),
            create_test_entry("medium.txt", FileType::File, 100),
        ];

        sort_entries(&mut entries, &SortColumn::Size, &SortOrder::Ascending);

        assert_eq!(entries[0].name, "small.txt");
        assert_eq!(entries[1].name, "medium.txt");
        assert_eq!(entries[2].name, "large.txt");
    }

    #[test]
    fn test_sort_by_size_descending() {
        let mut entries = vec![
            create_test_entry("large.txt", FileType::File, 1000),
            create_test_entry("small.txt", FileType::File, 10),
            create_test_entry("medium.txt", FileType::File, 100),
        ];

        sort_entries(&mut entries, &SortColumn::Size, &SortOrder::Descending);

        assert_eq!(entries[0].name, "large.txt");
        assert_eq!(entries[1].name, "medium.txt");
        assert_eq!(entries[2].name, "small.txt");
    }

    #[test]
    fn test_sort_directories_before_files() {
        let mut entries = vec![
            create_test_entry("file_a.txt", FileType::File, 100),
            create_test_entry("dir_z", FileType::Directory, 0),
            create_test_entry("file_b.txt", FileType::File, 100),
            create_test_entry("dir_a", FileType::Directory, 0),
        ];

        sort_entries(&mut entries, &SortColumn::Name, &SortOrder::Ascending);

        // Directories should come first, sorted by name
        assert_eq!(entries[0].name, "dir_a");
        assert_eq!(entries[1].name, "dir_z");
        // Then files, sorted by name
        assert_eq!(entries[2].name, "file_a.txt");
        assert_eq!(entries[3].name, "file_b.txt");
    }

    #[test]
    fn test_sort_parent_entry_always_first() {
        let mut entries = vec![
            create_test_entry("apple.txt", FileType::File, 100),
            FileEntry::parent_entry(PathBuf::from("/parent")),
            create_test_entry("aaa", FileType::Directory, 0),
        ];

        sort_entries(&mut entries, &SortColumn::Name, &SortOrder::Ascending);

        // ".." should always be first
        assert_eq!(entries[0].name, "..");
        // Then directories
        assert_eq!(entries[1].name, "aaa");
        // Then files
        assert_eq!(entries[2].name, "apple.txt");
    }

    #[test]
    fn test_sort_by_extension() {
        let mut entries = vec![
            create_test_entry("file.txt", FileType::File, 100),
            create_test_entry("file.rs", FileType::File, 100),
            create_test_entry("file.doc", FileType::File, 100),
        ];

        sort_entries(&mut entries, &SortColumn::Extension, &SortOrder::Ascending);

        assert_eq!(entries[0].extension, Some("doc".to_string()));
        assert_eq!(entries[1].extension, Some("rs".to_string()));
        assert_eq!(entries[2].extension, Some("txt".to_string()));
    }

    // ==================== SortColumn and SortOrder Tests ====================

    #[test]
    fn test_sort_column_equality() {
        assert_eq!(SortColumn::Name, SortColumn::Name);
        assert_eq!(SortColumn::Size, SortColumn::Size);
        assert_eq!(SortColumn::Modified, SortColumn::Modified);
        assert_eq!(SortColumn::Extension, SortColumn::Extension);
        assert_ne!(SortColumn::Name, SortColumn::Size);
    }

    #[test]
    fn test_sort_order_equality() {
        assert_eq!(SortOrder::Ascending, SortOrder::Ascending);
        assert_eq!(SortOrder::Descending, SortOrder::Descending);
        assert_ne!(SortOrder::Ascending, SortOrder::Descending);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_sort_empty_entries() {
        let mut entries: Vec<FileEntry> = vec![];
        sort_entries(&mut entries, &SortColumn::Name, &SortOrder::Ascending);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_sort_single_entry() {
        let mut entries = vec![create_test_entry("single.txt", FileType::File, 100)];
        sort_entries(&mut entries, &SortColumn::Name, &SortOrder::Ascending);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "single.txt");
    }

    #[test]
    fn test_file_entry_nonexistent_path() {
        let result = FileEntry::new(PathBuf::from("/nonexistent/path/file.txt"));
        assert!(result.is_err());
    }
}
