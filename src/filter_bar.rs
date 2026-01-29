use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, ActiveTheme, IconName, Sizable};

pub struct FilterBar {
    filter_text: SharedString,
    visible: bool,
    focus_handle: FocusHandle,
}

pub enum FilterBarEvent {
    FilterChanged(String),
    FilterCleared,
    Dismissed,
}

impl EventEmitter<FilterBarEvent> for FilterBar {}

impl FilterBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            filter_text: SharedString::default(),
            visible: false,
            focus_handle,
        }
    }

    pub fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        self.filter_text = SharedString::default();
        cx.emit(FilterBarEvent::FilterCleared);
        cx.emit(FilterBarEvent::Dismissed);
        cx.notify();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.filter_text = SharedString::default();
        cx.emit(FilterBarEvent::FilterCleared);
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match &event.keystroke.key {
            key if key == "backspace" => {
                let mut text = self.filter_text.to_string();
                text.pop();
                self.filter_text = SharedString::from(text.clone());
                cx.emit(FilterBarEvent::FilterChanged(text));
                cx.notify();
            }
            key if key == "escape" => {
                self.hide(cx);
            }
            key if key.len() == 1 => {
                let ch = key.chars().next().unwrap();
                if ch.is_alphanumeric() || ch == ' ' || ch == '.' || ch == '_' || ch == '-' {
                    let mut text = self.filter_text.to_string();
                    text.push(ch);
                    self.filter_text = SharedString::from(text.clone());
                    cx.emit(FilterBarEvent::FilterChanged(text));
                    cx.notify();
                }
            }
            _ => {}
        }
    }
}

impl Render for FilterBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let filter_display = if self.filter_text.is_empty() {
            "Type to filter...".to_string()
        } else {
            self.filter_text.to_string()
        };

        let text_color = if self.filter_text.is_empty() {
            cx.theme().muted_foreground
        } else {
            cx.theme().foreground
        };

        h_flex()
            .id("filter-bar")
            .w_full()
            .h_8()
            .px_2()
            .gap_2()
            .items_center()
            .bg(cx.theme().secondary)
            .border_b_1()
            .border_color(cx.theme().border)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(
                gpui_component::Icon::new(IconName::Search)
                    .size_4()
                    .text_color(cx.theme().muted_foreground),
            )
            .child(
                div()
                    .flex_1()
                    .px_2()
                    .py_1()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().primary)
                    .rounded_md()
                    .text_sm()
                    .text_color(text_color)
                    .child(filter_display),
            )
            .child(
                Button::new("clear-filter")
                    .icon(IconName::Close)
                    .ghost()
                    .xsmall()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.hide(cx);
                    })),
            )
            .into_any_element()
    }
}

impl Focusable for FilterBar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
