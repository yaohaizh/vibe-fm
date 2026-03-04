use gpui::*;
use gpui_component::dialog::{Dialog, DialogButtonProps};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName};

pub struct ProgressDialog {
    visible: bool,
    title: String,
    current_file: String,
    progress: f64,
    total_items: usize,
    completed_items: usize,
    total_bytes: u64,
    completed_bytes: u64,
    is_cancelled: bool,
    focus_handle: FocusHandle,
}

pub enum ProgressDialogEvent {
    Completed,
    Cancelled,
}

impl EventEmitter<ProgressDialogEvent> for ProgressDialog {}

impl ProgressDialog {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            title: "Operation in Progress".to_string(),
            current_file: String::new(),
            progress: 0.0,
            total_items: 0,
            completed_items: 0,
            total_bytes: 0,
            completed_bytes: 0,
            is_cancelled: false,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn show(
        &mut self,
        title: &str,
        total_items: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.title = title.to_string();
        self.total_items = total_items;
        self.completed_items = 0;
        self.progress = 0.0;
        self.is_cancelled = false;
        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        cx.notify();
    }

    pub fn update_progress(
        &mut self,
        current_file: &str,
        completed_items: usize,
        completed_bytes: u64,
        cx: &mut Context<Self>,
    ) {
        self.current_file = current_file.to_string();
        self.completed_items = completed_items;
        self.completed_bytes = completed_bytes;

        if self.total_items > 0 {
            self.progress = (completed_items as f64 / self.total_items as f64) * 100.0;
        }

        cx.notify();
    }

    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    fn format_bytes(bytes: u64) -> String {
        humansize::format_size(bytes, humansize::BINARY)
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        self.is_cancelled = true;
        self.hide(cx);
        cx.emit(ProgressDialogEvent::Cancelled);
    }
}

impl Render for ProgressDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let progress_width = (self.progress / 100.0 * 300.0) as u32;

        Dialog::new(_window, cx)
            .title(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Loader)
                    .child(self.title.clone()),
            )
            .w(px(400.))
            .button_props(
                DialogButtonProps::default()
                    .ok_text("Cancel")
                    .cancel_text("Cancel"),
            )
            .on_cancel({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |dialog, cx| {
                        dialog.cancel(cx);
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
                            .child(format!(
                                "{} of {} items",
                                self.completed_items, self.total_items
                            )),
                    )
                    .child(
                        div()
                            .h_3()
                            .w_full()
                            .bg(cx.theme().secondary)
                            .rounded_md()
                            .overflow_hidden()
                            .child(
                                div()
                                    .h_full()
                                    .w(px(progress_width as f32))
                                    .bg(cx.theme().primary),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .truncate()
                            .child(format!("{}", self.current_file)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{} processed",
                                Self::format_bytes(self.completed_bytes)
                            )),
                    ),
            )
            .into_any_element()
    }
}

impl Focusable for ProgressDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
