use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, ActiveTheme, IconName, Sizable};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelSide {
    Left,
    Right,
}

pub struct DriveSelector {
    drives: Vec<DriveInfo>,
    panel_side: PanelSide,
}

#[derive(Clone, Debug)]
pub struct DriveInfo {
    pub path: PathBuf,
    pub name: String,
    pub label: Option<String>,
}

pub enum DriveSelectorEvent {
    DriveSelected(PathBuf, PanelSide),
}

impl EventEmitter<DriveSelectorEvent> for DriveSelector {}

impl DriveSelector {
    pub fn new(panel_side: PanelSide) -> Self {
        let drives = Self::get_drives();
        Self { drives, panel_side }
    }

    #[cfg(target_os = "windows")]
    fn get_drives() -> Vec<DriveInfo> {
        let mut drives = Vec::new();

        for letter in b'A'..=b'Z' {
            let drive_letter = letter as char;
            let path = PathBuf::from(format!("{}:\\", drive_letter));
            if path.exists() {
                drives.push(DriveInfo {
                    path: path.clone(),
                    name: format!("{}:", drive_letter),
                    label: None,
                });
            }
        }

        drives
    }

    #[cfg(not(target_os = "windows"))]
    fn get_drives() -> Vec<DriveInfo> {
        let mut drives = Vec::new();

        drives.push(DriveInfo {
            path: PathBuf::from("/"),
            name: "/".to_string(),
            label: Some("Root".to_string()),
        });

        if let Some(home) = dirs::home_dir() {
            drives.push(DriveInfo {
                path: home,
                name: "~".to_string(),
                label: Some("Home".to_string()),
            });
        }

        let mount_points = ["/mnt", "/media", "/Volumes"];
        for mount in mount_points {
            let mount_path = PathBuf::from(mount);
            if mount_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&mount_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            drives.push(DriveInfo {
                                path,
                                name: name.clone(),
                                label: Some(name),
                            });
                        }
                    }
                }
            }
        }

        drives
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.drives = Self::get_drives();
        cx.notify();
    }
}

impl Render for DriveSelector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel_side = self.panel_side;

        h_flex()
            .gap_1()
            .px_3()
            .py_2()
            .flex_wrap()
            .bg(cx.theme().background)
            .border_b_1()
            .border_color(cx.theme().border)
            .children(self.drives.iter().map(|drive| {
                let path = drive.path.clone();
                let display_name = drive.label.clone().unwrap_or_else(|| drive.name.clone());

                Button::new(SharedString::from(format!(
                    "drive-{}-{}",
                    drive.name, panel_side as u8
                )))
                .child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(IconName::FolderOpen)
                        .child(display_name),
                )
                .ghost()
                .small()
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.emit(DriveSelectorEvent::DriveSelected(path.clone(), panel_side));
                }))
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::{DriveInfo, DriveSelector, PanelSide};
    use std::path::PathBuf;

    // ==================== PanelSide Tests ====================

    #[test]
    fn test_panel_side_equality() {
        assert_eq!(PanelSide::Left, PanelSide::Left);
        assert_eq!(PanelSide::Right, PanelSide::Right);
        assert_ne!(PanelSide::Left, PanelSide::Right);
    }

    #[test]
    fn test_panel_side_copy() {
        let side = PanelSide::Left;
        let copied = side;
        assert_eq!(side, copied);
    }

    // ==================== DriveInfo Tests ====================

    #[test]
    fn test_drive_info_creation() {
        let drive = DriveInfo {
            path: PathBuf::from("C:\\"),
            name: "C:".to_string(),
            label: Some("Local Disk".to_string()),
        };

        assert_eq!(drive.path, PathBuf::from("C:\\"));
        assert_eq!(drive.name, "C:");
        assert_eq!(drive.label, Some("Local Disk".to_string()));
    }

    #[test]
    fn test_drive_info_without_label() {
        let drive = DriveInfo {
            path: PathBuf::from("/"),
            name: "/".to_string(),
            label: None,
        };

        assert_eq!(drive.path, PathBuf::from("/"));
        assert_eq!(drive.name, "/");
        assert!(drive.label.is_none());
    }

    #[test]
    fn test_drive_info_clone() {
        let drive = DriveInfo {
            path: PathBuf::from("/home"),
            name: "~".to_string(),
            label: Some("Home".to_string()),
        };

        let cloned = drive.clone();
        assert_eq!(cloned.path, drive.path);
        assert_eq!(cloned.name, drive.name);
        assert_eq!(cloned.label, drive.label);
    }

    // ==================== get_drives Tests ====================

    #[test]
    fn test_get_drives_returns_non_empty() {
        let drives = DriveSelector::get_drives();
        // On any system, there should be at least one drive/mount point
        assert!(!drives.is_empty());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_get_drives_windows_format() {
        let drives = DriveSelector::get_drives();

        // All Windows drives should have names in format "X:"
        for drive in &drives {
            assert!(drive.name.len() == 2);
            assert!(drive.name.ends_with(':'));
            assert!(drive.name.chars().next().unwrap().is_ascii_uppercase());
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_get_drives_unix_has_root() {
        let drives = DriveSelector::get_drives();

        // Unix systems should always have root
        let has_root = drives.iter().any(|d| d.path == PathBuf::from("/"));
        assert!(has_root);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_get_drives_unix_has_home() {
        let drives = DriveSelector::get_drives();

        // Should have home directory if it exists
        if dirs::home_dir().is_some() {
            let has_home = drives.iter().any(|d| d.name == "~");
            assert!(has_home);
        }
    }
}
