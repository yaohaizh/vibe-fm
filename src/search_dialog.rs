use gpui::*;
use gpui_component::dialog::{Dialog, DialogButtonProps};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct SearchDialog {
    visible: bool,
    search_path: PathBuf,
    search_term: String,
    case_sensitive: bool,
    search_subdirs: bool,
    results: Vec<SearchResult>,
    focus_handle: FocusHandle,
}

#[derive(Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

pub enum SearchDialogEvent {
    Closed,
    FileSelected(PathBuf),
}

impl EventEmitter<SearchDialogEvent> for SearchDialog {}

impl SearchDialog {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            search_path: PathBuf::new(),
            search_term: String::new(),
            case_sensitive: false,
            search_subdirs: true,
            results: Vec::new(),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn show(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.search_path = path.clone();
        self.search_term = String::new();
        self.results.clear();
        self.visible = true;
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        cx.notify();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_search_term(&mut self, term: String) {
        self.search_term = term;
    }

    pub fn set_case_sensitive(&mut self, value: bool) {
        self.case_sensitive = value;
    }

    pub fn set_search_subdirs(&mut self, value: bool) {
        self.search_subdirs = value;
    }

    pub fn perform_search(&mut self) {
        if self.search_term.is_empty() {
            return;
        }

        self.results.clear();

        let search_term = if self.case_sensitive {
            self.search_term.clone()
        } else {
            self.search_term.to_lowercase()
        };

        let max_results = 500;

        let walker = if self.search_subdirs {
            WalkDir::new(&self.search_path).into_iter()
        } else {
            WalkDir::new(&self.search_path).max_depth(1).into_iter()
        };

        for entry in walker.filter_map(|e| e.ok()) {
            if self.results.len() >= max_results {
                break;
            }

            let file_name = entry.file_name().to_string_lossy();
            let matches = if self.case_sensitive {
                file_name.contains(&search_term)
            } else {
                file_name.to_lowercase().contains(&search_term)
            };

            if matches {
                let metadata = entry.metadata().ok();
                let is_dir = entry.file_type().is_dir();
                let size = metadata.map(|m| m.len()).unwrap_or(0);

                self.results.push(SearchResult {
                    path: entry.path().to_path_buf(),
                    name: file_name.to_string(),
                    is_dir,
                    size,
                });
            }
        }
    }
}

impl Render for SearchDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let results = self.results.clone();

        Dialog::new(_window, cx)
            .title(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Search)
                    .child("Search Files"),
            )
            .w(px(600.))
            .h(px(500.))
            .button_props(
                DialogButtonProps::default()
                    .ok_text("Close")
                    .cancel_text("Cancel"),
            )
            .on_cancel({
                let this = cx.entity().clone();
                move |_, _, cx| {
                    let _ = this.update(cx, |dialog, cx| {
                        dialog.hide(cx);
                        cx.emit(SearchDialogEvent::Closed);
                    });
                    true
                }
            })
            .child(
                v_flex()
                    .p_4()
                    .gap_4()
                    .size_full()
                    .child(
                        div()
                            .text_sm()
                            .child(format!("Search in: {}", self.search_path.display())),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Type search term and press Enter to search"),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(format!("Results: {} found", results.len()))
                    .child(
                        div()
                            .flex_1()
                            .size_full()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_md()
                            .overflow_y_scrollbar()
                            .children(results.iter().map(|result| {
                                div().p_2().text_color(cx.theme().foreground).child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .w_full()
                                        .child(if result.is_dir {
                                            IconName::Folder
                                        } else {
                                            IconName::File
                                        })
                                        .child(result.name.clone())
                                        .child(div().flex_1())
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format_size(result.size)),
                                        ),
                                )
                            })),
                    ),
            )
            .into_any_element()
    }
}

impl Focusable for SearchDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
