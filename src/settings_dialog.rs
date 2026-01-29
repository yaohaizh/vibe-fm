use crate::settings::{AppSettings, DateFormat, SizeFormat, SortColumn, SortOrder};
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::scroll::ScrollableElement;
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

    pub fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Store original settings so we can revert on cancel
        self.original_settings = self.settings.clone();
        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn update_settings(&mut self, settings: AppSettings, cx: &mut Context<Self>) {
        self.settings = settings.clone();
        self.original_settings = settings;
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
}

impl Render for SettingsDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

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

        // Fixed header
        let header = h_flex()
            .w_full()
            .flex_shrink_0()
            .px_4()
            .py_3()
            .justify_between()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        gpui_component::Icon::new(IconName::Settings)
                            .size_5()
                            .text_color(cx.theme().primary),
                    )
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().foreground)
                            .child("Settings"),
                    ),
            )
            .child(
                Button::new("close-settings")
                    .icon(IconName::Close)
                    .ghost()
                    .small()
                    .tooltip("Close (Esc)")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.cancel(cx);
                    })),
            );

        // Scrollable content area with visible scrollbar
        let scrollable_content = div()
            .id("settings-scroll-area")
            .flex_1()
            .overflow_y_scrollbar()
            .child(
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
                        |this, checked, cx| {
                            // Placeholder for future feature
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::render_checkbox_row(
                        "show-preview",
                        "Show file preview pane",
                        false,
                        |this, checked, cx| {
                            // Placeholder for future feature
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::render_checkbox_row(
                        "auto-refresh",
                        "Auto-refresh directory listing",
                        true,
                        |this, checked, cx| {
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
                        |this, value, cx| {
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
                        |this, value, cx| {
                            // Placeholder for future feature
                            cx.notify();
                        },
                        cx,
                    ))
                    // Add some bottom padding for better scrolling experience
                    .child(div().h_8()),
            );

        // Fixed footer with OK/Cancel buttons
        let footer = h_flex()
            .w_full()
            .flex_shrink_0()
            .px_4()
            .py_3()
            .justify_end()
            .gap_3()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary)
            .child(
                Button::new("cancel-settings")
                    .label("Cancel")
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.cancel(cx);
                    })),
            )
            .child(
                Button::new("ok-settings")
                    .label("OK")
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.save_and_close(cx);
                    })),
            );

        // Combine into dialog content with proper flex layout
        let dialog_content = v_flex()
            .size_full()
            .child(header)
            .child(scrollable_content)
            .child(footer);

        // The overlay container
        // The dialog is centered using flexbox
        div()
            .id("settings-overlay")
            .absolute()
            .inset_0()
            .flex()
            .justify_center()
            .items_start()
            .pt(px(80.))
            .bg(rgba(0x000000A0))
            .on_click(cx.listener(|this, _, _, cx| {
                // This will be called for clicks on the backdrop only
                // because the dialog has its own click handler that stops propagation
                this.cancel(cx);
            }))
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(
                // Dialog box - centered, with click handler to stop propagation
                div()
                    .id("settings-dialog-box")
                    .w(px(520.))
                    .max_h(px(400.))
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .shadow_xl()
                    .on_click(cx.listener(|_, _, _, cx| {
                        // Stop propagation - prevent backdrop click handler
                        cx.stop_propagation();
                    }))
                    .child(dialog_content),
            )
            .into_any_element()
    }
}

impl Focusable for SettingsDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
