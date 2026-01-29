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
            .px_2()
            .py_1()
            .flex_wrap()
            .bg(cx.theme().secondary)
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
