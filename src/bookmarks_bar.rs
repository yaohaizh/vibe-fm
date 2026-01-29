use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single bookmark entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
}

impl Bookmark {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }
}

/// Bookmarks data that can be persisted to disk
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BookmarksData {
    pub bookmarks: Vec<Bookmark>,
}

impl BookmarksData {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("vibe-fm").join("bookmarks.json"))
    }

    pub fn load() -> Self {
        if let Some(config_path) = Self::config_path() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(data) = serde_json::from_str(&content) {
                    return data;
                }
            }
        }
        // Return default bookmarks if no saved data
        Self::default_bookmarks()
    }

    pub fn save(&self) {
        if let Some(config_path) = Self::config_path() {
            if let Some(parent) = config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(&config_path, content);
            }
        }
    }

    fn default_bookmarks() -> Self {
        let mut bookmarks = vec![];

        // Add common default locations
        if let Some(home) = dirs::home_dir() {
            bookmarks.push(Bookmark::new("Home".to_string(), home.clone()));

            if let Some(docs) = dirs::document_dir() {
                bookmarks.push(Bookmark::new("Documents".to_string(), docs));
            }

            if let Some(downloads) = dirs::download_dir() {
                bookmarks.push(Bookmark::new("Downloads".to_string(), downloads));
            }

            if let Some(desktop) = dirs::desktop_dir() {
                bookmarks.push(Bookmark::new("Desktop".to_string(), desktop));
            }

            #[cfg(target_os = "windows")]
            {
                bookmarks.push(Bookmark::new("C: Drive".to_string(), PathBuf::from("C:\\")));
            }

            #[cfg(not(target_os = "windows"))]
            {
                bookmarks.push(Bookmark::new("Root".to_string(), PathBuf::from("/")));
            }
        }

        Self { bookmarks }
    }
}

pub struct BookmarksBar {
    visible: bool,
    bookmarks: BookmarksData,
}

pub enum BookmarksBarEvent {
    BookmarkSelected(PathBuf),
    Dismissed,
}

impl EventEmitter<BookmarksBarEvent> for BookmarksBar {}

impl BookmarksBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            visible: false,
            bookmarks: BookmarksData::load(),
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.visible = !self.visible;
        cx.notify();
    }

    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.visible = true;
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        cx.emit(BookmarksBarEvent::Dismissed);
        cx.notify();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn add_bookmark(&mut self, name: String, path: PathBuf, cx: &mut Context<Self>) {
        // Don't add duplicates
        if self.bookmarks.bookmarks.iter().any(|b| b.path == path) {
            return;
        }
        self.bookmarks.bookmarks.push(Bookmark::new(name, path));
        self.bookmarks.save();
        cx.notify();
    }

    pub fn remove_bookmark(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.bookmarks.bookmarks.len() {
            self.bookmarks.bookmarks.remove(index);
            self.bookmarks.save();
            cx.notify();
        }
    }

    fn get_icon_for_bookmark(bookmark: &Bookmark) -> IconName {
        let path = &bookmark.path;
        let name_lower = bookmark.name.to_lowercase();

        // Special icons based on name
        if name_lower.contains("home") {
            return IconName::Folder;
        }
        if name_lower.contains("document") {
            return IconName::File;
        }
        if name_lower.contains("download") {
            return IconName::Inbox;
        }
        if name_lower.contains("desktop") {
            return IconName::Frame;
        }
        if name_lower.contains("drive") || path.parent().is_none() {
            return IconName::FolderOpen;
        }
        if name_lower.contains("music") {
            return IconName::Star;
        }
        if name_lower.contains("picture") || name_lower.contains("photo") {
            return IconName::GalleryVerticalEnd;
        }
        if name_lower.contains("video") || name_lower.contains("movie") {
            return IconName::Star;
        }

        IconName::Folder
    }
}

impl Render for BookmarksBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        let bookmarks: Vec<(usize, Bookmark)> = self
            .bookmarks
            .bookmarks
            .iter()
            .cloned()
            .enumerate()
            .collect();

        v_flex()
            .id("bookmarks-bar")
            .w(px(200.0))
            .h_full()
            .flex_shrink_0()
            .bg(cx.theme().secondary)
            .border_r_1()
            .border_color(cx.theme().border)
            .overflow_y_scroll()
            // Header
            .child(
                h_flex()
                    .w_full()
                    .h_8()
                    .px_2()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(
                                gpui_component::Icon::new(IconName::Star)
                                    .size_4()
                                    .text_color(cx.theme().primary),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Bookmarks"),
                            ),
                    )
                    .child(
                        Button::new("close-bookmarks")
                            .icon(IconName::Close)
                            .ghost()
                            .xsmall()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.hide(cx);
                            })),
                    ),
            )
            // Bookmark list
            .child(
                v_flex()
                    .w_full()
                    .flex_1()
                    .p_1()
                    .gap_px()
                    .children(bookmarks.into_iter().map(|(index, bookmark)| {
                        let path = bookmark.path.clone();
                        let icon = Self::get_icon_for_bookmark(&bookmark);
                        let hover_bg = cx.theme().accent;

                        h_flex()
                            .id(ElementId::Name(format!("bookmark-{}", index).into()))
                            .w_full()
                            .h_7()
                            .px_2()
                            .gap_2()
                            .items_center()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|style| style.bg(hover_bg))
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                cx.emit(BookmarksBarEvent::BookmarkSelected(path.clone()));
                            }))
                            .child(
                                gpui_component::Icon::new(icon)
                                    .size_4()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_ellipsis()
                                    .overflow_hidden()
                                    .text_color(cx.theme().foreground)
                                    .child(bookmark.name.clone()),
                            )
                    })),
            )
            .into_any_element()
    }
}
