use crate::settings::{AppSettings, DateFormat, SizeFormat, SortColumn, SortOrder};
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::dialog::{Dialog, DialogButtonProps};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable};

pub struct SettingsDialog {
    visible: bool,
    settings: AppSettings,
    original_settings: AppSettings,
    focus_handle: FocusHandle,
}

pub enum SettingsDialogEvent {
    Saved(AppSettings),
    Cancelled,
}

impl EventEmitter<SettingsDialogEvent> for SettingsDialog {}

impl SettingsDialog {
    pub fn new(settings: AppSettings, cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            settings: settings.clone(),
            original_settings: settings,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn show(&mut self, settings: AppSettings, window: &mut Window, cx: &mut Context<Self>) {
        // Update settings and store original so we can revert on cancel
        self.settings = settings.clone();
        self.original_settings = settings;
        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    fn save_and_close(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        cx.emit(SettingsDialogEvent::Saved(self.settings.clone()));
        cx.notify();
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        // Revert to original settings
        self.settings = self.original_settings.clone();
        self.visible = false;
        cx.emit(SettingsDialogEvent::Cancelled);
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key == "escape" {
            self.cancel(cx);
        }
    }

    fn render_section_header(title: &str, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .pt_4()
            .pb_2()
            .mb_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(cx.theme().primary)
                    .child(title.to_string()),
            )
    }

    fn render_checkbox_row(
        id: &'static str,
        label: &str,
        checked: bool,
        on_change: impl Fn(&mut Self, bool, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .w_full()
            .py_1()
            .px_2()
            .gap_3()
            .items_center()
            .rounded_md()
            .hover(|style| style.bg(cx.theme().secondary))
            .child(Checkbox::new(id).checked(checked).on_click(cx.listener(
                move |this, checked: &bool, _, cx| {
                    on_change(this, *checked, cx);
                },
            )))
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(label.to_string()),
            )
    }

    fn render_option_row<T: Clone + PartialEq + 'static>(
        label: &str,
        current: T,
        options: Vec<(T, &'static str, &'static str)>,
        on_change: impl Fn(&mut Self, T, &mut Context<Self>) + Clone + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut buttons = h_flex().gap_1();

        for (value, id, display) in options {
            let is_selected = value == current;
            let on_change = on_change.clone();
            let value_clone = value.clone();

            let btn = Button::new(id).label(display).small();
            let btn = if is_selected {
                btn.primary().on_click(cx.listener(move |_, _, _, _| {}))
            } else {
                btn.ghost().on_click(cx.listener(move |this, _, _, cx| {
                    on_change(this, value_clone.clone(), cx);
                }))
            };
            buttons = buttons.child(btn);
        }

        h_flex()
            .w_full()
            .py_1()
            .px_2()
            .gap_3()
            .items_center()
            .justify_between()
            .rounded_md()
            .hover(|style| style.bg(cx.theme().secondary))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(label.to_string()),
            )
            .child(buttons)
    }

    fn render_settings_content(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Get current values for rendering
        let show_hidden = self.settings.show_hidden_files;
        let show_extensions = self.settings.show_file_extensions;
        let confirm_delete = self.settings.confirm_before_delete;
        let single_click = self.settings.single_click_to_open;
        let remember_paths = self.settings.remember_last_paths;
        let date_format = self.settings.date_format;
        let size_format = self.settings.size_format;
        let sort_column = self.settings.default_sort_column;
        let sort_order = self.settings.default_sort_order;

        v_flex()
            .p_4()
            .gap_1()
            // Display Settings Section
            .child(Self::render_section_header("Display", cx))
            .child(Self::render_checkbox_row(
                "show-hidden",
                "Show hidden files",
                show_hidden,
                |this, checked, cx| {
                    this.settings.show_hidden_files = checked;
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_checkbox_row(
                "show-extensions",
                "Show file extensions",
                show_extensions,
                |this, checked, cx| {
                    this.settings.show_file_extensions = checked;
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_option_row(
                "Date format",
                date_format,
                vec![
                    (DateFormat::YmdHm, "date-ymd", "YYYY-MM-DD"),
                    (DateFormat::DmyHm, "date-dmy", "DD/MM/YYYY"),
                    (DateFormat::MdyHm, "date-mdy", "MM/DD/YYYY"),
                ],
                |this, value, cx| {
                    this.settings.date_format = value;
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_option_row(
                "Size format",
                size_format,
                vec![
                    (SizeFormat::Binary, "size-binary", "Binary (KiB)"),
                    (SizeFormat::Decimal, "size-decimal", "Decimal (KB)"),
                ],
                |this, value, cx| {
                    this.settings.size_format = value;
                    cx.notify();
                },
                cx,
            ))
            // Behavior Settings Section
            .child(Self::render_section_header("Behavior", cx))
            .child(Self::render_checkbox_row(
                "confirm-delete",
                "Confirm before delete",
                confirm_delete,
                |this, checked, cx| {
                    this.settings.confirm_before_delete = checked;
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_checkbox_row(
                "single-click",
                "Single click to open files",
                single_click,
                |this, checked, cx| {
                    this.settings.single_click_to_open = checked;
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_option_row(
                "Default sort",
                sort_column,
                vec![
                    (SortColumn::Name, "sort-name", "Name"),
                    (SortColumn::Size, "sort-size", "Size"),
                    (SortColumn::Modified, "sort-modified", "Modified"),
                ],
                |this, value, cx| {
                    this.settings.default_sort_column = value;
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_option_row(
                "Sort order",
                sort_order,
                vec![
                    (SortOrder::Ascending, "order-asc", "Ascending"),
                    (SortOrder::Descending, "order-desc", "Descending"),
                ],
                |this, value, cx| {
                    this.settings.default_sort_order = value;
                    cx.notify();
                },
                cx,
            ))
            // Panel Settings Section
            .child(Self::render_section_header("Panels", cx))
            .child(Self::render_checkbox_row(
                "remember-paths",
                "Remember last opened paths",
                remember_paths,
                |this, checked, cx| {
                    this.settings.remember_last_paths = checked;
                    cx.notify();
                },
                cx,
            ))
            // Advanced Settings Section
            .child(Self::render_section_header("Advanced", cx))
            .child(Self::render_checkbox_row(
                "show-thumbnails",
                "Show image thumbnails",
                false,
                |_this, _checked, cx| {
                    // Placeholder for future feature
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_checkbox_row(
                "show-preview",
                "Show file preview pane",
                false,
                |_this, _checked, cx| {
                    // Placeholder for future feature
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_checkbox_row(
                "auto-refresh",
                "Auto-refresh directory listing",
                true,
                |_this, _checked, cx| {
                    // Placeholder for future feature
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_option_row(
                "Theme",
                "light",
                vec![
                    ("light", "theme-light", "Light"),
                    ("dark", "theme-dark", "Dark"),
                    ("auto", "theme-auto", "Auto"),
                ],
                |_this, _value, cx| {
                    // Placeholder for future feature
                    cx.notify();
                },
                cx,
            ))
            .child(Self::render_option_row(
                "Language",
                "en",
                vec![
                    ("en", "lang-en", "English"),
                    ("fr", "lang-fr", "French"),
                    ("de", "lang-de", "German"),
                    ("es", "lang-es", "Spanish"),
                ],
                |_this, _value, cx| {
                    // Placeholder for future feature
                    cx.notify();
                },
                cx,
            ))
    }
}

impl Render for SettingsDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        // Use gpui-component's Dialog for a native dialog experience
        Dialog::new(window, cx)
            .title(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Settings)
                    .child("Settings"),
            )
            .w(px(560.))
            .confirm()
            .button_props(
                DialogButtonProps::default()
                    .ok_text("Save")
                    .cancel_text("Cancel"),
            )
            .on_ok({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |dialog, cx| {
                        dialog.save_and_close(cx);
                    });
                    true // Close the dialog
                }
            })
            .on_cancel({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |dialog, cx| {
                        dialog.cancel(cx);
                    });
                    true // Close the dialog
                }
            })
            .child(self.render_settings_content(cx))
            .into_any_element()
    }
}

impl Focusable for SettingsDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
