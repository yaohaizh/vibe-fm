use crate::file_entry::FileEntry;
use crate::file_ops;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::dialog::{Dialog, DialogButtonProps};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName};
use std::path::PathBuf;

pub struct RenameDialog {
    visible: bool,
    entry: Option<FileEntry>,
    new_name: String,
    is_directory: bool,
    error_message: Option<String>,
    focus_handle: FocusHandle,
}

pub enum RenameDialogEvent {
    Renamed(PathBuf, String),
    Cancelled,
}

impl EventEmitter<RenameDialogEvent> for RenameDialog {}

impl RenameDialog {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            entry: None,
            new_name: String::new(),
            is_directory: false,
            error_message: None,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn show(&mut self, entry: FileEntry, window: &mut Window, cx: &mut Context<Self>) {
        self.entry = Some(entry.clone());
        self.new_name = entry.name.clone();
        self.is_directory = entry.is_directory();
        self.error_message = None;
        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        self.entry = None;
        self.new_name.clear();
        self.error_message = None;
        cx.notify();
    }

    fn validate_and_rename(&mut self, cx: &mut Context<Self>) {
        let entry = match &self.entry {
            Some(e) => e,
            None => return,
        };

        let new_name = self.new_name.trim();
        if new_name.is_empty() {
            self.error_message = Some("Name cannot be empty".to_string());
            cx.notify();
            return;
        }

        if new_name.contains('/') || new_name.contains('\\') {
            self.error_message = Some("Name cannot contain / or \\".to_string());
            cx.notify();
            return;
        }

        let parent = entry.path.parent().unwrap_or(&entry.path);
        let new_path = parent.join(new_name);

        if new_path.exists() && new_path != entry.path {
            self.error_message = Some("A file or folder with this name already exists".to_string());
            cx.notify();
            return;
        }

        match file_ops::rename_item(&entry.path, new_name) {
            Ok(final_path) => {
                let old_name = entry.name.clone();
                self.hide(cx);
                cx.emit(RenameDialogEvent::Renamed(final_path, old_name));
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to rename: {}", e));
                cx.notify();
            }
        }
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key == "escape" {
            self.hide(cx);
            cx.emit(RenameDialogEvent::Cancelled);
        }
    }
}

impl Render for RenameDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let current_name = self
            .entry
            .as_ref()
            .map(|e| e.name.clone())
            .unwrap_or_default();
        let is_dir = self.is_directory;

        Dialog::new(_window, cx)
            .title(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(if is_dir {
                        IconName::Folder
                    } else {
                        IconName::File
                    })
                    .child(if is_dir {
                        "Rename Folder"
                    } else {
                        "Rename File"
                    }),
            )
            .w(px(420.))
            .confirm()
            .button_props(
                DialogButtonProps::default()
                    .ok_text("Rename")
                    .cancel_text("Cancel"),
            )
            .on_ok({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |dialog, cx| {
                        dialog.validate_and_rename(cx);
                    });
                    true
                }
            })
            .on_cancel({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |dialog, cx| {
                        dialog.hide(cx);
                        cx.emit(RenameDialogEvent::Cancelled);
                    });
                    true
                }
            })
            .child(
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(cx.theme().foreground)
                                    .child("Current name:"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .truncate()
                                    .child(current_name.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(cx.theme().foreground)
                            .child("New name:"),
                    )
                    .child(
                        div()
                            .w_full()
                            .h_8()
                            .px_2()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_md()
                            .bg(cx.theme().background)
                            .text_color(cx.theme().foreground)
                            .child(self.new_name.clone()),
                    )
                    .when_some(self.error_message.as_ref(), |el, msg| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().accent)
                                .child(msg.clone()),
                        )
                    }),
            )
            .into_any_element()
    }
}

impl Focusable for RenameDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
