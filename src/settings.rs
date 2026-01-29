use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Sort column options for file listing
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortColumn {
    #[default]
    Name,
    Size,
    Modified,
}

impl SortColumn {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "size" => SortColumn::Size,
            "modified" => SortColumn::Modified,
            _ => SortColumn::Name,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            SortColumn::Name => "name",
            SortColumn::Size => "size",
            SortColumn::Modified => "modified",
        }
    }
}

/// Sort order options
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

impl SortOrder {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "descending" | "desc" => SortOrder::Descending,
            _ => SortOrder::Ascending,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "ascending",
            SortOrder::Descending => "descending",
        }
    }
}

/// File size display format
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SizeFormat {
    #[default]
    Binary, // KiB, MiB, GiB (1024-based)
    Decimal, // KB, MB, GB (1000-based)
}

impl SizeFormat {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "decimal" => SizeFormat::Decimal,
            _ => SizeFormat::Binary,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            SizeFormat::Binary => "binary",
            SizeFormat::Decimal => "decimal",
        }
    }
}

/// Date format options
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateFormat {
    #[default]
    YmdHm, // YYYY-MM-DD HH:MM
    DmyHm,    // DD/MM/YYYY HH:MM
    MdyHm,    // MM/DD/YYYY HH:MM
    Relative, // "2 hours ago", "Yesterday", etc.
}

impl DateFormat {
    pub fn format_string(&self) -> &'static str {
        match self {
            DateFormat::YmdHm => "%Y-%m-%d %H:%M",
            DateFormat::DmyHm => "%d/%m/%Y %H:%M",
            DateFormat::MdyHm => "%m/%d/%Y %H:%M",
            DateFormat::Relative => "", // Handled specially
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DateFormat::YmdHm => "YYYY-MM-DD HH:MM",
            DateFormat::DmyHm => "DD/MM/YYYY HH:MM",
            DateFormat::MdyHm => "MM/DD/YYYY HH:MM",
            DateFormat::Relative => "Relative (2 hours ago)",
        }
    }

    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dmy" | "dmyhm" => DateFormat::DmyHm,
            "mdy" | "mdyhm" => DateFormat::MdyHm,
            "relative" => DateFormat::Relative,
            _ => DateFormat::YmdHm,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            DateFormat::YmdHm => "ymd",
            DateFormat::DmyHm => "dmy",
            DateFormat::MdyHm => "mdy",
            DateFormat::Relative => "relative",
        }
    }
}

/// Application settings
#[derive(Clone, Debug)]
pub struct AppSettings {
    // Display settings
    pub show_hidden_files: bool,
    pub show_file_extensions: bool,
    pub date_format: DateFormat,
    pub size_format: SizeFormat,

    // Behavior settings
    pub confirm_before_delete: bool,
    pub single_click_to_open: bool,
    pub default_sort_column: SortColumn,
    pub default_sort_order: SortOrder,

    // Panel settings
    pub remember_last_paths: bool,
    pub left_panel_path: Option<String>,
    pub right_panel_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Display settings
            show_hidden_files: false,
            show_file_extensions: true,
            date_format: DateFormat::default(),
            size_format: SizeFormat::default(),

            // Behavior settings
            confirm_before_delete: true,
            single_click_to_open: false,
            default_sort_column: SortColumn::default(),
            default_sort_order: SortOrder::default(),

            // Panel settings
            remember_last_paths: true,
            left_panel_path: None,
            right_panel_path: None,
        }
    }
}

/// Simple INI parser
struct IniFile {
    sections: HashMap<String, HashMap<String, String>>,
}

impl IniFile {
    fn new() -> Self {
        Self {
            sections: HashMap::new(),
        }
    }

    fn parse(content: &str) -> Self {
        let mut ini = Self::new();
        let mut current_section = String::from("General");

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                continue;
            }

            // Key=Value pair
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();

                ini.sections
                    .entry(current_section.clone())
                    .or_insert_with(HashMap::new)
                    .insert(key, value);
            }
        }

        ini
    }

    fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section).and_then(|s| s.get(key))
    }

    fn get_bool(&self, section: &str, key: &str, default: bool) -> bool {
        self.get(section, key)
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "yes" | "1"))
            .unwrap_or(default)
    }

    fn get_string(&self, section: &str, key: &str) -> Option<String> {
        self.get(section, key).cloned()
    }

    fn set(&mut self, section: &str, key: &str, value: &str) {
        self.sections
            .entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }

    fn to_string(&self) -> String {
        let mut output = String::new();

        // Define section order for consistent output
        let section_order = ["Display", "Behavior", "Panels"];

        for section_name in &section_order {
            if let Some(section) = self.sections.get(*section_name) {
                output.push_str(&format!("[{}]\n", section_name));

                // Sort keys for consistent output
                let mut keys: Vec<_> = section.keys().collect();
                keys.sort();

                for key in keys {
                    if let Some(value) = section.get(key) {
                        output.push_str(&format!("{}={}\n", key, value));
                    }
                }
                output.push('\n');
            }
        }

        // Handle any sections not in the predefined order
        for (section_name, section) in &self.sections {
            if section_order.contains(&section_name.as_str()) {
                continue;
            }

            output.push_str(&format!("[{}]\n", section_name));

            let mut keys: Vec<_> = section.keys().collect();
            keys.sort();

            for key in keys {
                if let Some(value) = section.get(key) {
                    output.push_str(&format!("{}={}\n", key, value));
                }
            }
            output.push('\n');
        }

        output
    }
}

impl AppSettings {
    /// Get the settings file path (fm.ini in config directory)
    fn settings_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("APPDATA")
                .ok()
                .map(|p| PathBuf::from(p).join("vibe-fm").join("fm.ini"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            dirs::config_dir().map(|p| p.join("vibe-fm").join("fm.ini"))
        }
    }

    /// Load settings from INI file, or return defaults if file doesn't exist
    pub fn load() -> Self {
        let path = match Self::settings_path() {
            Some(p) => p,
            None => {
                log::warn!("Could not determine settings path, using defaults");
                return Self::default();
            }
        };

        if !path.exists() {
            log::info!("Settings file not found at {:?}, using defaults", path);
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => {
                let ini = IniFile::parse(&contents);
                let settings = Self::from_ini(&ini);
                log::info!("Settings loaded from {:?}", path);
                settings
            }
            Err(e) => {
                log::error!("Failed to read settings file: {}", e);
                Self::default()
            }
        }
    }

    fn from_ini(ini: &IniFile) -> Self {
        Self {
            // Display section
            show_hidden_files: ini.get_bool("Display", "show_hidden_files", false),
            show_file_extensions: ini.get_bool("Display", "show_file_extensions", true),
            date_format: ini
                .get_string("Display", "date_format")
                .map(|s| DateFormat::from_str(&s))
                .unwrap_or_default(),
            size_format: ini
                .get_string("Display", "size_format")
                .map(|s| SizeFormat::from_str(&s))
                .unwrap_or_default(),

            // Behavior section
            confirm_before_delete: ini.get_bool("Behavior", "confirm_before_delete", true),
            single_click_to_open: ini.get_bool("Behavior", "single_click_to_open", false),
            default_sort_column: ini
                .get_string("Behavior", "default_sort_column")
                .map(|s| SortColumn::from_str(&s))
                .unwrap_or_default(),
            default_sort_order: ini
                .get_string("Behavior", "default_sort_order")
                .map(|s| SortOrder::from_str(&s))
                .unwrap_or_default(),

            // Panels section
            remember_last_paths: ini.get_bool("Panels", "remember_last_paths", true),
            left_panel_path: ini
                .get_string("Panels", "left_panel_path")
                .filter(|s| !s.is_empty()),
            right_panel_path: ini
                .get_string("Panels", "right_panel_path")
                .filter(|s| !s.is_empty()),
        }
    }

    fn to_ini(&self) -> IniFile {
        let mut ini = IniFile::new();

        // Display section
        ini.set(
            "Display",
            "show_hidden_files",
            if self.show_hidden_files {
                "true"
            } else {
                "false"
            },
        );
        ini.set(
            "Display",
            "show_file_extensions",
            if self.show_file_extensions {
                "true"
            } else {
                "false"
            },
        );
        ini.set("Display", "date_format", self.date_format.as_str());
        ini.set("Display", "size_format", self.size_format.as_str());

        // Behavior section
        ini.set(
            "Behavior",
            "confirm_before_delete",
            if self.confirm_before_delete {
                "true"
            } else {
                "false"
            },
        );
        ini.set(
            "Behavior",
            "single_click_to_open",
            if self.single_click_to_open {
                "true"
            } else {
                "false"
            },
        );
        ini.set(
            "Behavior",
            "default_sort_column",
            self.default_sort_column.as_str(),
        );
        ini.set(
            "Behavior",
            "default_sort_order",
            self.default_sort_order.as_str(),
        );

        // Panels section
        ini.set(
            "Panels",
            "remember_last_paths",
            if self.remember_last_paths {
                "true"
            } else {
                "false"
            },
        );
        ini.set(
            "Panels",
            "left_panel_path",
            self.left_panel_path.as_deref().unwrap_or(""),
        );
        ini.set(
            "Panels",
            "right_panel_path",
            self.right_panel_path.as_deref().unwrap_or(""),
        );

        ini
    }

    /// Save settings to INI file
    pub fn save(&self) -> Result<(), String> {
        let path = match Self::settings_path() {
            Some(p) => p,
            None => return Err("Could not determine settings path".to_string()),
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return Err(format!("Failed to create settings directory: {}", e));
                }
            }
        }

        let ini = self.to_ini();
        let content = format!(
            "; Vibe File Manager Configuration\n; This file is auto-generated. Edit with care.\n\n{}",
            ini.to_string()
        );

        match fs::write(&path, content) {
            Ok(_) => {
                log::info!("Settings saved to {:?}", path);
                Ok(())
            }
            Err(e) => Err(format!("Failed to write settings file: {}", e)),
        }
    }

    /// Get the path where settings are stored (for display purposes)
    pub fn get_settings_path() -> Option<PathBuf> {
        Self::settings_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert!(!settings.show_hidden_files);
        assert!(settings.show_file_extensions);
        assert!(settings.confirm_before_delete);
        assert!(!settings.single_click_to_open);
        assert!(settings.remember_last_paths);
    }

    #[test]
    fn test_ini_parse_basic() {
        let content = r#"
[Display]
show_hidden_files=true
show_file_extensions=false

[Behavior]
confirm_before_delete=false
"#;
        let ini = IniFile::parse(content);
        assert_eq!(ini.get_bool("Display", "show_hidden_files", false), true);
        assert_eq!(ini.get_bool("Display", "show_file_extensions", true), false);
        assert_eq!(
            ini.get_bool("Behavior", "confirm_before_delete", true),
            false
        );
    }

    #[test]
    fn test_ini_roundtrip() {
        let mut settings = AppSettings::default();
        settings.show_hidden_files = true;
        settings.date_format = DateFormat::DmyHm;
        settings.left_panel_path = Some("C:\\Users".to_string());

        let ini = settings.to_ini();
        let content = ini.to_string();
        let parsed_ini = IniFile::parse(&content);
        let loaded = AppSettings::from_ini(&parsed_ini);

        assert_eq!(settings.show_hidden_files, loaded.show_hidden_files);
        assert_eq!(settings.date_format, loaded.date_format);
        assert_eq!(settings.left_panel_path, loaded.left_panel_path);
    }

    #[test]
    fn test_date_format_conversion() {
        assert_eq!(DateFormat::from_str("ymd"), DateFormat::YmdHm);
        assert_eq!(DateFormat::from_str("dmy"), DateFormat::DmyHm);
        assert_eq!(DateFormat::from_str("mdy"), DateFormat::MdyHm);
        assert_eq!(DateFormat::from_str("relative"), DateFormat::Relative);
    }

    #[test]
    fn test_sort_column_conversion() {
        assert_eq!(SortColumn::from_str("name"), SortColumn::Name);
        assert_eq!(SortColumn::from_str("size"), SortColumn::Size);
        assert_eq!(SortColumn::from_str("modified"), SortColumn::Modified);
    }

    #[test]
    fn test_sort_order_conversion() {
        assert_eq!(SortOrder::from_str("ascending"), SortOrder::Ascending);
        assert_eq!(SortOrder::from_str("descending"), SortOrder::Descending);
        assert_eq!(SortOrder::from_str("desc"), SortOrder::Descending);
    }

    #[test]
    fn test_date_format_display_names() {
        assert_eq!(DateFormat::YmdHm.display_name(), "YYYY-MM-DD HH:MM");
        assert_eq!(DateFormat::DmyHm.display_name(), "DD/MM/YYYY HH:MM");
        assert_eq!(DateFormat::MdyHm.display_name(), "MM/DD/YYYY HH:MM");
    }

    #[test]
    fn test_date_format_strings() {
        assert_eq!(DateFormat::YmdHm.format_string(), "%Y-%m-%d %H:%M");
        assert_eq!(DateFormat::DmyHm.format_string(), "%d/%m/%Y %H:%M");
    }
}
