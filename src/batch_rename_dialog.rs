use crate::file_entry::FileEntry;
use crate::file_ops;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{Dialog, DialogButtonProps};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum BatchRenameMode {
    Replace,
    PrefixSuffix,
    Case,
    Numbering,
}

pub struct BatchRenameDialog {
    visible: bool,
    entries: Vec<FileEntry>,
    mode: BatchRenameMode,
    find_text: String,
    replace_text: String,
    prefix: String,
    suffix: String,
    case_mode: String,
    start_number: u32,
    padding: u32,
    focus_handle: FocusHandle,
}

pub enum BatchRenameDialogEvent {
    Renamed,
    Cancelled,
}

impl EventEmitter<BatchRenameDialogEvent> for BatchRenameDialog {}

impl BatchRenameDialog {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            entries: Vec::new(),
            mode: BatchRenameMode::Replace,
            find_text: String::new(),
            replace_text: String::new(),
            prefix: String::new(),
            suffix: String::new(),
            case_mode: "lowercase".to_string(),
            start_number: 1,
            padding: 1,
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

    fn apply_rename(&mut self, cx: &mut Context<Self>) {
        let mut errors = Vec::new();

        for (index, entry) in self.entries.iter().enumerate() {
            let new_name = match self.mode {
                BatchRenameMode::Replace => {
                    if self.find_text.is_empty() {
                        entry.name.clone()
                    } else {
                        entry.name.replace(&self.find_text, &self.replace_text)
                    }
                }
                BatchRenameMode::PrefixSuffix => {
                    format!("{}{}{}", self.prefix, entry.name, self.suffix)
                }
                BatchRenameMode::Case => match self.case_mode.as_str() {
                    "uppercase" => entry.name.to_uppercase(),
                    "lowercase" => entry.name.to_lowercase(),
                    "titlecase" => {
                        let mut chars = entry.name.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>()
                                    + &chars.as_str().to_lowercase()
                            }
                        }
                    }
                    _ => entry.name.clone(),
                },
                BatchRenameMode::Numbering => {
                    let ext = entry.extension.clone();
                    let stem = entry
                        .name
                        .rsplit_once('.')
                        .map(|(s, _)| s)
                        .unwrap_or(&entry.name);

                    let num = self.start_number + index as u32;
                    let num_str = format!("{:0>width$}", num, width = self.padding as usize);

                    if let Some(ext) = ext {
                        format!("{}_{}.{}", stem, num_str, ext)
                    } else {
                        format!("{}_{}", stem, num_str)
                    }
                }
            };

            if new_name != entry.name && !new_name.is_empty() {
                match file_ops::rename_item(&entry.path, &new_name) {
                    Err(e) => {
                        errors.push(format!("{}: {}", entry.name, e));
                    }
                    _ => {}
                }
            }
        }

        if errors.is_empty() {
            self.hide(cx);
            cx.emit(BatchRenameDialogEvent::Renamed);
        } else {
            log::error!("Batch rename errors: {:?}", errors);
            self.hide(cx);
            cx.emit(BatchRenameDialogEvent::Renamed);
        }
    }
}

impl Render for BatchRenameDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let entries_count = self.entries.len();

        let mode_buttons = h_flex()
            .gap_1()
            .child(
                Button::new("mode-replace")
                    .label("Replace")
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mode = BatchRenameMode::Replace;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("mode-prefix")
                    .label("Prefix/Suffix")
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mode = BatchRenameMode::PrefixSuffix;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("mode-case")
                    .label("Case")
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mode = BatchRenameMode::Case;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("mode-number")
                    .label("Numbering")
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mode = BatchRenameMode::Numbering;
                        cx.notify();
                    })),
            );

        let options = match self.mode {
            BatchRenameMode::Replace => v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w(px(80.))
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Find:"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(self.find_text.clone()),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w(px(80.))
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Replace:"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(self.replace_text.clone()),
                        ),
                ),
            BatchRenameMode::PrefixSuffix => v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w(px(80.))
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Prefix:"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(self.prefix.clone()),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w(px(80.))
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Suffix:"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(self.suffix.clone()),
                        ),
                ),
            BatchRenameMode::Case => v_flex().gap_2().child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .w(px(80.))
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child("Mode:"),
                    )
                    .child(
                        Button::new("case-lower")
                            .label("lowercase")
                            .small()
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.case_mode = "lowercase".to_string();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("case-upper")
                            .label("UPPERCASE")
                            .small()
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.case_mode = "uppercase".to_string();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("case-title")
                            .label("Title Case")
                            .small()
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.case_mode = "titlecase".to_string();
                                cx.notify();
                            })),
                    ),
            ),
            BatchRenameMode::Numbering => v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w(px(80.))
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Start:"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(format!("{}", self.start_number)),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w(px(80.))
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("Padding:"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(format!("{} digits", self.padding)),
                        ),
                ),
        };

        Dialog::new(_window, cx)
            .title(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Settings)
                    .child("Batch Rename"),
            )
            .w(px(500.))
            .button_props(
                DialogButtonProps::default()
                    .ok_text("Rename")
                    .cancel_text("Cancel"),
            )
            .on_ok({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |dialog, cx| {
                        dialog.apply_rename(cx);
                    });
                    true
                }
            })
            .on_cancel({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |dialog, cx| {
                        dialog.hide(cx);
                        cx.emit(BatchRenameDialogEvent::Cancelled);
                    });
                    true
                }
            })
            .child(
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(format!("Renaming {} items", entries_count)),
                    )
                    .child(mode_buttons)
                    .child(options),
            )
            .into_any_element()
    }
}

impl Focusable for BatchRenameDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
