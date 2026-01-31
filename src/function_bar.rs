use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, ActiveTheme, Sizable};

pub struct FunctionBar;

pub enum FunctionBarAction {
    View,      // F3
    Edit,      // F4
    Copy,      // F5
    Move,      // F6
    NewFolder, // F7
    Delete,    // F8
    Rename,    // F9 (Shift+F6 in some file managers)
    Exit,      // F10/Alt+F4
}

pub enum FunctionBarEvent {
    Action(FunctionBarAction),
}

impl EventEmitter<FunctionBarEvent> for FunctionBar {}

impl FunctionBar {
    pub fn new() -> Self {
        Self
    }
}

impl Render for FunctionBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .h_9()
            .bg(cx.theme().background)
            .border_t_1()
            .border_color(cx.theme().border)
            .items_center()
            .justify_between()
            .px_2()
            .gap_1()
            // F3 - View
            .child(
                Button::new("fn-f3")
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("F3"),
                            )
                            .child("View"),
                    )
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(FunctionBarEvent::Action(FunctionBarAction::View));
                    })),
            )
            // F4 - Edit
            .child(
                Button::new("fn-f4")
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("F4"),
                            )
                            .child("Edit"),
                    )
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(FunctionBarEvent::Action(FunctionBarAction::Edit));
                    })),
            )
            // F5 - Copy
            .child(
                Button::new("fn-f5")
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("F5"),
                            )
                            .child("Copy"),
                    )
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(FunctionBarEvent::Action(FunctionBarAction::Copy));
                    })),
            )
            // F6 - Move
            .child(
                Button::new("fn-f6")
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("F6"),
                            )
                            .child("Move"),
                    )
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(FunctionBarEvent::Action(FunctionBarAction::Move));
                    })),
            )
            // F7 - New Folder
            .child(
                Button::new("fn-f7")
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("F7"),
                            )
                            .child("NewDir"),
                    )
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(FunctionBarEvent::Action(FunctionBarAction::NewFolder));
                    })),
            )
            // F8 - Delete
            .child(
                Button::new("fn-f8")
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("F8"),
                            )
                            .child("Delete"),
                    )
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(FunctionBarEvent::Action(FunctionBarAction::Delete));
                    })),
            )
            // F9 - Rename (classic Shift+F6 in some)
            .child(
                Button::new("fn-f9")
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("F9"),
                            )
                            .child("Rename"),
                    )
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(FunctionBarEvent::Action(FunctionBarAction::Rename));
                    })),
            )
            // F10/Alt+X - Exit
            .child(
                Button::new("fn-f10")
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("F10"),
                            )
                            .child("Exit"),
                    )
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(FunctionBarEvent::Action(FunctionBarAction::Exit));
                    })),
            )
    }
}
