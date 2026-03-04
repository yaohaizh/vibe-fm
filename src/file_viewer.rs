use gpui::*;
use gpui_component::dialog::{Dialog, DialogButtonProps};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme};
use std::fs;
use std::path::PathBuf;

pub struct FileViewer {
    visible: bool,
    file_path: PathBuf,
    content: String,
    file_size: u64,
    focus_handle: FocusHandle,
}

pub enum FileViewerEvent {
    Closed,
}

impl EventEmitter<FileViewerEvent> for FileViewer {}

impl FileViewer {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            file_path: PathBuf::new(),
            content: String::new(),
            file_size: 0,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn show(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.file_path = path.clone();

        let max_size = 1024 * 1024;

        if let Ok(metadata) = fs::metadata(&path) {
            self.file_size = metadata.len();

            if self.file_size <= max_size {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        self.content = content;
                    }
                    Err(_) => {
                        self.content = "[Binary or unsupported file - cannot display]".to_string();
                    }
                }
            } else {
                self.content = format!(
                    "[File too large to display ({} bytes)]\n\nShowing first 1MB...\n\n{}",
                    self.file_size,
                    fs::read_to_string(&path)
                        .map(|c| c.chars().take(1024 * 1024).collect::<String>())
                        .unwrap_or_default()
                );
            }
        }

        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        self.content.clear();
        cx.notify();
    }
}

impl Render for FileViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let file_name = self
            .file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "File Viewer".to_string());

        Dialog::new(_window, cx)
            .title(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(format!("Viewing: {}", file_name)),
            )
            .w(px(700.))
            .h(px(500.))
            .button_props(
                DialogButtonProps::default()
                    .ok_text("Close")
                    .cancel_text("Cancel"),
            )
            .on_cancel({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    this.update(cx, |viewer, cx| {
                        viewer.hide(cx);
                        cx.emit(FileViewerEvent::Closed);
                    });
                    true
                }
            })
            .child(
                v_flex()
                    .flex_1()
                    .size_full()
                    .p_2()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex_1()
                            .size_full()
                            .overflow_y_scrollbar()
                            .font_family("monospace")
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .children(
                                self.content
                                    .lines()
                                    .map(|line| div().w_full().child(line.to_string())),
                            ),
                    ),
            )
            .into_any_element()
    }
}

impl Focusable for FileViewer {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
