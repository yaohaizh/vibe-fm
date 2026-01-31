use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{h_flex, ActiveTheme};

pub struct StatusBar {
    left_path: String,
    right_path: String,
    left_selected: usize,
    right_selected: usize,
    left_total: usize,
    right_total: usize,
    active_panel: ActivePanel,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Left,
    Right,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            left_path: String::new(),
            right_path: String::new(),
            left_selected: 0,
            right_selected: 0,
            left_total: 0,
            right_total: 0,
            active_panel: ActivePanel::Left,
        }
    }

    pub fn update_left(
        &mut self,
        path: String,
        selected: usize,
        total: usize,
        cx: &mut Context<Self>,
    ) {
        self.left_path = path;
        self.left_selected = selected;
        self.left_total = total;
        cx.notify();
    }

    pub fn update_right(
        &mut self,
        path: String,
        selected: usize,
        total: usize,
        cx: &mut Context<Self>,
    ) {
        self.right_path = path;
        self.right_selected = selected;
        self.right_total = total;
        cx.notify();
    }

    pub fn set_active_panel(&mut self, panel: ActivePanel, cx: &mut Context<Self>) {
        self.active_panel = panel;
        cx.notify();
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let left_status = if self.left_selected > 0 {
            format!("{}/{} selected", self.left_selected, self.left_total)
        } else {
            format!("{} items", self.left_total)
        };

        let right_status = if self.right_selected > 0 {
            format!("{}/{} selected", self.right_selected, self.right_total)
        } else {
            format!("{} items", self.right_total)
        };

        h_flex()
            .h_7()
            .px_4()
            .justify_between()
            .items_center()
            .bg(cx.theme().background)
            .border_t_1()
            .border_color(cx.theme().border)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(
                h_flex().gap_4().flex_1().child(
                    div()
                        .when(self.active_panel == ActivePanel::Left, |el: Div| {
                            el.text_color(cx.theme().foreground)
                                .font_weight(FontWeight::MEDIUM)
                        })
                        .child(format!("Left: {}", left_status)),
                ),
            )
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child("Vibe File Manager v0.1.0"),
            )
            .child(
                h_flex().gap_4().flex_1().justify_end().child(
                    div()
                        .when(self.active_panel == ActivePanel::Right, |el: Div| {
                            el.text_color(cx.theme().foreground)
                                .font_weight(FontWeight::MEDIUM)
                        })
                        .child(format!("Right: {}", right_status)),
                ),
            )
    }
}
