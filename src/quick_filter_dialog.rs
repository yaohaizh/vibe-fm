use gpui::*;
use gpui_component::dialog::{Dialog, DialogButtonProps};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName};

pub struct QuickFilterDialog {
    visible: bool,
    filter_text: String,
    focus_handle: FocusHandle,
}

pub enum QuickFilterDialogEvent {
    Closed,
    FilterApplied(String),
}

impl EventEmitter<QuickFilterDialogEvent> for QuickFilterDialog {}

impl QuickFilterDialog {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            filter_text: String::new(),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter_text = String::new();
        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        cx.notify();
    }
}

impl Render for QuickFilterDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let filter_text = self.filter_text.clone();

        Dialog::new(_window, cx)
            .title(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Search)
                    .child("Quick Filter"),
            )
            .w(px(350.))
            .button_props(
                DialogButtonProps::default()
                    .ok_text("Apply")
                    .cancel_text("Cancel"),
            )
            .on_cancel({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    let _ = this.update(cx, |dialog, cx| {
                        dialog.hide(cx);
                        cx.emit(QuickFilterDialogEvent::Closed);
                    });
                    true
                }
            })
            .on_ok({
                let this = cx.entity().clone();
                let filter_text = filter_text.clone();
                move |_, _, cx| {
                    let _ = this.update(cx, |dialog, cx| {
                        dialog.hide(cx);
                        cx.emit(QuickFilterDialogEvent::FilterApplied(filter_text.clone()));
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
                            .text_color(cx.theme().muted_foreground)
                            .child("Enter filter pattern (e.g., *.txt, *.rs, data*)"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Leave empty to show all files"),
                    )
                    .child(
                        div()
                            .p_2()
                            .rounded_md()
                            .bg(cx.theme().secondary)
                            .text_color(cx.theme().foreground)
                            .text_sm()
                            .children(vec![
                                div().child("* - Show all"),
                                div().child("*.ext - Files with extension"),
                                div().child("name* - Starts with name"),
                                div().child("*name* - Contains name"),
                            ]),
                    ),
            )
            .into_any_element()
    }
}

impl Focusable for QuickFilterDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
