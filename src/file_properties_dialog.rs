use crate::file_entry::FileEntry;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::dialog::{Dialog, DialogButtonProps};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName};

pub struct FilePropertiesDialog {
    visible: bool,
    entries: Vec<FileEntry>,
    focus_handle: FocusHandle,
}

pub enum FilePropertiesDialogEvent {
    Closed,
}

impl EventEmitter<FilePropertiesDialogEvent> for FilePropertiesDialog {}

impl FilePropertiesDialog {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            entries: Vec::new(),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn show(&mut self, entries: Vec<FileEntry>, window: &mut Window, cx: &mut Context<Self>) {
        self.entries = entries;
        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        self.entries.clear();
        cx.notify();
    }

    fn format_size(size: u64) -> String {
        humansize::format_size(size, humansize::BINARY)
    }

    fn format_date(time: Option<std::time::SystemTime>) -> String {
        time.map(|t| {
            let datetime: chrono::DateTime<chrono::Local> = t.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| "N/A".to_string())
    }
}

impl Render for FilePropertiesDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let entries_count = self.entries.len();
        let title = if entries_count == 1 {
            let entry = &self.entries[0];
            if entry.is_directory() {
                "Folder Properties"
            } else {
                "File Properties"
            }
        } else {
            "Properties"
        };

        let total_size: u64 = self.entries.iter().map(|e| e.size).sum();

        let content = if entries_count == 1 {
            let entry = &self.entries[0];
            v_flex()
                .p_4()
                .gap_3()
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            h_flex()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Name:"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(cx.theme().foreground)
                                        .child(entry.name.clone()),
                                ),
                        )
                        .child(
                            h_flex()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Type:"),
                                )
                                .child(div().text_sm().text_color(cx.theme().foreground).child(
                                    if entry.is_directory() {
                                        "Folder"
                                    } else {
                                        "File"
                                    },
                                )),
                        )
                        .child(
                            h_flex()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Location:"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().foreground)
                                        .truncate()
                                        .child(
                                            entry
                                                .path
                                                .parent()
                                                .map(|p| p.to_string_lossy().to_string())
                                                .unwrap_or_else(|| "N/A".to_string()),
                                        ),
                                ),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(cx.theme().foreground)
                                .child("Time"),
                        )
                        .child(
                            h_flex()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Modified:"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().foreground)
                                        .child(Self::format_date(entry.modified)),
                                ),
                        ),
                )
        } else {
            v_flex()
                .p_4()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child(format!("{} items selected", entries_count)),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Total size:"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(cx.theme().foreground)
                                .child(Self::format_size(total_size)),
                        ),
                )
        };

        Dialog::new(_window, cx)
            .title(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Info)
                    .child(title),
            )
            .w(px(450.))
            .button_props(
                DialogButtonProps::default()
                    .ok_text("Close")
                    .cancel_text("Cancel"),
            )
            .on_cancel({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |dialog, cx| {
                        dialog.hide(cx);
                        cx.emit(FilePropertiesDialogEvent::Closed);
                    });
                    true
                }
            })
            .child(content)
            .into_any_element()
    }
}

impl Focusable for FilePropertiesDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
