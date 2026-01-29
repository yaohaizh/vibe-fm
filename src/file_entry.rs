use chrono::{DateTime, Local};
use gpui_component::IconName;
use humansize::{format_size, BINARY};
use std::path::PathBuf;
use std::time::SystemTime;

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

    pub fn formatted_date(&self) -> String {
        self.modified
            .map(|t| {
                let datetime: DateTime<Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_default()
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
