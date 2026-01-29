use crate::file_entry::{sort_entries, FileEntry, SortColumn, SortOrder};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::breadcrumb::{Breadcrumb, BreadcrumbItem};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::ContextMenuExt;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{
    h_flex, v_flex, ActiveTheme, Disableable, IconName, InteractiveElementExt, Selectable, Sizable,
};
use std::path::PathBuf;

// Define actions for panel-specific keyboard navigation
actions!(
    file_panel,
    [MoveUp, MoveDown, MoveToStart, MoveToEnd, PageUp, PageDown,]
);

/// Register global key bindings for file panel navigation
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, Some("FilePanel")),
        KeyBinding::new("down", MoveDown, Some("FilePanel")),
        KeyBinding::new("home", MoveToStart, Some("FilePanel")),
        KeyBinding::new("end", MoveToEnd, Some("FilePanel")),
        KeyBinding::new("pageup", PageUp, Some("FilePanel")),
        KeyBinding::new("pagedown", PageDown, Some("FilePanel")),
    ]);
}

pub struct FilePanel {
    id: &'static str,
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    selected_indices: Vec<usize>,
    last_selected_index: Option<usize>, // For shift+click range selection
    sort_column: SortColumn,
    sort_order: SortOrder,
    show_hidden: bool,
    history: Vec<PathBuf>,
    history_index: usize,
    is_active: bool,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
}

pub enum FilePanelEvent {
    PathChanged(PathBuf),
    SelectionChanged(Vec<FileEntry>),
    FileOpened(PathBuf),
    DirectoryEntered(PathBuf),
    PanelFocused,
    // Context menu actions
    RequestCopy,
    RequestCut,
    RequestPaste,
    RequestDelete,
    RequestNewFolder,
    RequestNewFile,
    RequestRename,
    RequestRefresh,
}

impl FilePanel {
    pub fn new(
        id: &'static str,
        initial_path: PathBuf,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let entries = Self::read_directory(&initial_path, true);
        let focus_handle = cx.focus_handle();

        Self {
            id,
            current_path: initial_path.clone(),
            entries,
            selected_indices: vec![],
            last_selected_index: None,
            sort_column: SortColumn::Name,
            sort_order: SortOrder::Ascending,
            show_hidden: true,
            history: vec![initial_path],
            history_index: 0,
            is_active: false,
            scroll_handle: ScrollHandle::new(),
            focus_handle,
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if path.is_dir() {
            self.history_index += 1;
            self.history.truncate(self.history_index);
            self.history.push(path.clone());

            self.current_path = path.clone();
            self.refresh(cx);

            cx.emit(FilePanelEvent::PathChanged(path.clone()));
            cx.emit(FilePanelEvent::DirectoryEntered(path));
        }
    }

    pub fn navigate_up(&mut self, cx: &mut Context<Self>) {
        if let Some(parent) = self.current_path.parent() {
            self.navigate_to(parent.to_path_buf(), cx);
        }
    }

    pub fn navigate_back(&mut self, cx: &mut Context<Self>) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(path) = self.history.get(self.history_index).cloned() {
                self.current_path = path.clone();
                self.refresh(cx);
                cx.emit(FilePanelEvent::PathChanged(path));
            }
        }
    }

    pub fn navigate_forward(&mut self, cx: &mut Context<Self>) {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            if let Some(path) = self.history.get(self.history_index).cloned() {
                self.current_path = path.clone();
                self.refresh(cx);
                cx.emit(FilePanelEvent::PathChanged(path));
            }
        }
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.entries = Self::read_directory(&self.current_path, self.show_hidden);
        sort_entries(&mut self.entries, &self.sort_column, &self.sort_order);
        self.selected_indices.clear();
        self.last_selected_index = None;
        cx.notify();
    }

    pub fn toggle_hidden(&mut self, cx: &mut Context<Self>) {
        self.show_hidden = !self.show_hidden;
        self.refresh(cx);
    }

    pub fn set_active(&mut self, active: bool, cx: &mut Context<Self>) {
        self.is_active = active;
        cx.notify();
    }

    pub fn current_path(&self) -> &PathBuf {
        &self.current_path
    }

    pub fn selected_entries(&self) -> Vec<FileEntry> {
        self.selected_indices
            .iter()
            .filter_map(|&i| self.entries.get(i).cloned())
            .collect()
    }

    fn select_single(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected_indices = vec![index];
        self.last_selected_index = Some(index);
        let selected = self.selected_entries();
        cx.emit(FilePanelEvent::SelectionChanged(selected));
        cx.notify();
    }

    /// Ctrl+Click: Toggle selection of a single item
    fn toggle_selection_ctrl(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.selected_indices.contains(&index) {
            self.selected_indices.retain(|&i| i != index);
        } else {
            self.selected_indices.push(index);
        }
        self.last_selected_index = Some(index);
        let selected = self.selected_entries();
        cx.emit(FilePanelEvent::SelectionChanged(selected));
        cx.notify();
    }

    /// Shift+Click: Range selection from last selected to current
    fn select_range(&mut self, index: usize, cx: &mut Context<Self>) {
        let start = self.last_selected_index.unwrap_or(0);
        let (from, to) = if start <= index {
            (start, index)
        } else {
            (index, start)
        };

        self.selected_indices = (from..=to).collect();
        // Don't update last_selected_index on range select to allow extending
        let selected = self.selected_entries();
        cx.emit(FilePanelEvent::SelectionChanged(selected));
        cx.notify();
    }

    /// Handle click with modifiers
    pub fn handle_click(&mut self, index: usize, modifiers: Modifiers, cx: &mut Context<Self>) {
        if modifiers.control {
            // Ctrl+Click: Toggle selection
            self.toggle_selection_ctrl(index, cx);
        } else if modifiers.shift {
            // Shift+Click: Range selection
            self.select_range(index, cx);
        } else {
            // Normal click: Single selection
            self.select_single(index, cx);
        }
    }

    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        // Select all entries except ".." (parent directory)
        self.selected_indices = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.name != "..")
            .map(|(i, _)| i)
            .collect();
        let selected = self.selected_entries();
        cx.emit(FilePanelEvent::SelectionChanged(selected));
        cx.notify();
    }

    pub fn open_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(&index) = self.selected_indices.first() {
            if let Some(entry) = self.entries.get(index) {
                if entry.is_directory() {
                    let path = entry.path.clone();
                    self.navigate_to(path, cx);
                } else {
                    cx.emit(FilePanelEvent::FileOpened(entry.path.clone()));
                }
            }
        }
    }

    pub fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        let current = self.selected_indices.first().copied().unwrap_or(0);
        let new_index = if current > 0 { current - 1 } else { 0 };
        self.select_single(new_index, cx);
    }

    pub fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        let current = self.selected_indices.first().copied().unwrap_or(0);
        let max_index = self.entries.len().saturating_sub(1);
        let new_index = if current < max_index {
            current + 1
        } else {
            max_index
        };
        self.select_single(new_index, cx);
    }

    pub fn move_selection_to_start(&mut self, cx: &mut Context<Self>) {
        if !self.entries.is_empty() {
            self.select_single(0, cx);
        }
    }

    pub fn move_selection_to_end(&mut self, cx: &mut Context<Self>) {
        if !self.entries.is_empty() {
            self.select_single(self.entries.len() - 1, cx);
        }
    }

    pub fn page_up(&mut self, cx: &mut Context<Self>) {
        let current = self.selected_indices.first().copied().unwrap_or(0);
        let page_size = 10; // Move by 10 items
        let new_index = current.saturating_sub(page_size);
        self.select_single(new_index, cx);
    }

    pub fn page_down(&mut self, cx: &mut Context<Self>) {
        let current = self.selected_indices.first().copied().unwrap_or(0);
        let page_size = 10;
        let max_index = self.entries.len().saturating_sub(1);
        let new_index = (current + page_size).min(max_index);
        self.select_single(new_index, cx);
    }

    // Action handlers
    fn handle_move_up(&mut self, _: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_active {
            self.move_selection_up(cx);
        }
    }

    fn handle_move_down(&mut self, _: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_active {
            self.move_selection_down(cx);
        }
    }

    fn handle_move_to_start(
        &mut self,
        _: &MoveToStart,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_active {
            self.move_selection_to_start(cx);
        }
    }

    fn handle_move_to_end(&mut self, _: &MoveToEnd, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_active {
            self.move_selection_to_end(cx);
        }
    }

    fn handle_page_up(&mut self, _: &PageUp, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_active {
            self.page_up(cx);
        }
    }

    fn handle_page_down(&mut self, _: &PageDown, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_active {
            self.page_down(cx);
        }
    }

    fn read_directory(path: &PathBuf, show_hidden: bool) -> Vec<FileEntry> {
        let mut entries = Vec::new();

        if let Some(parent) = path.parent() {
            entries.push(FileEntry::parent_entry(parent.to_path_buf()));
        }

        if let Ok(read_dir) = std::fs::read_dir(path) {
            for entry in read_dir.flatten() {
                if let Ok(file_entry) = FileEntry::new(entry.path()) {
                    if show_hidden || !file_entry.is_hidden {
                        entries.push(file_entry);
                    }
                }
            }
        }

        entries.sort_by(|a, b| {
            if a.name == ".." {
                return std::cmp::Ordering::Less;
            }
            if b.name == ".." {
                return std::cmp::Ordering::Greater;
            }
            match (&a.file_type, &b.file_type) {
                (
                    crate::file_entry::FileType::Directory,
                    crate::file_entry::FileType::Directory,
                ) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                (crate::file_entry::FileType::Directory, _) => std::cmp::Ordering::Less,
                (_, crate::file_entry::FileType::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        entries
    }

    fn render_navigation_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let can_go_back = self.history_index > 0;
        let can_go_forward = self.history_index < self.history.len() - 1;
        let can_go_up = self.current_path.parent().is_some();

        // Build breadcrumb items from path components
        let entity = cx.entity().clone();
        let breadcrumb = self.build_breadcrumb(entity);

        h_flex()
            .gap_1()
            .p_1()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary)
            .child(
                Button::new("back")
                    .icon(IconName::ArrowLeft)
                    .ghost()
                    .compact()
                    .small()
                    .disabled(!can_go_back)
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.navigate_back(cx);
                    })),
            )
            .child(
                Button::new("forward")
                    .icon(IconName::ArrowRight)
                    .ghost()
                    .compact()
                    .small()
                    .disabled(!can_go_forward)
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.navigate_forward(cx);
                    })),
            )
            .child(
                Button::new("up")
                    .icon(IconName::ArrowUp)
                    .ghost()
                    .compact()
                    .small()
                    .disabled(!can_go_up)
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.navigate_up(cx);
                    })),
            )
            .child(
                Button::new("refresh")
                    .icon(IconName::Loader)
                    .ghost()
                    .compact()
                    .small()
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.refresh(cx);
                    })),
            )
            .child(div().w_2())
            .child(
                div()
                    .flex_1()
                    .h_7()
                    .px_2()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_md()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .child(breadcrumb),
            )
            .child(div().w_1())
            .child(
                Button::new("hidden")
                    .icon(if self.show_hidden {
                        IconName::Eye
                    } else {
                        IconName::EyeOff
                    })
                    .ghost()
                    .compact()
                    .small()
                    .selected(self.show_hidden)
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.toggle_hidden(cx);
                    })),
            )
    }

    fn build_breadcrumb(&self, entity: Entity<Self>) -> Breadcrumb {
        let mut breadcrumb = Breadcrumb::new();

        // Get path components
        let path = &self.current_path;
        let mut accumulated_path = PathBuf::new();

        // On Windows, handle the drive letter specially
        #[cfg(target_os = "windows")]
        {
            if let Some(prefix) = path.components().next() {
                let prefix_str = prefix.as_os_str().to_string_lossy().to_string();
                accumulated_path.push(&prefix_str);
                let click_path = accumulated_path.clone();
                let entity_clone = entity.clone();
                breadcrumb =
                    breadcrumb.child(BreadcrumbItem::new(prefix_str).on_click(move |_, _, cx| {
                        entity_clone.update(cx, |panel, cx| {
                            panel.navigate_to(click_path.clone(), cx);
                        });
                    }));
            }
        }

        // On Unix, start with root
        #[cfg(not(target_os = "windows"))]
        {
            accumulated_path.push("/");
            let entity_clone = entity.clone();
            breadcrumb = breadcrumb.child(BreadcrumbItem::new("/").on_click(move |_, _, cx| {
                entity_clone.update(cx, |panel, cx| {
                    panel.navigate_to(PathBuf::from("/"), cx);
                });
            }));
        }

        // Add remaining path components
        for component in path.components().skip(1) {
            let component_str = component.as_os_str().to_string_lossy().to_string();
            accumulated_path.push(&component_str);
            let click_path = accumulated_path.clone();
            let entity_clone = entity.clone();

            breadcrumb = breadcrumb.child(BreadcrumbItem::new(component_str).on_click(
                move |_, _, cx| {
                    entity_clone.update(cx, |panel, cx| {
                        panel.navigate_to(click_path.clone(), cx);
                    });
                },
            ));
        }

        breadcrumb
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current_sort_column = self.sort_column.clone();
        let current_sort_order = self.sort_order.clone();

        h_flex()
            .h_7()
            .px_2()
            .bg(cx.theme().secondary)
            .border_b_1()
            .border_color(cx.theme().border)
            .items_center()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(cx.theme().muted_foreground)
            .child(div().w(px(24.)))
            .child(self.render_sortable_header(
                "Name",
                SortColumn::Name,
                &current_sort_column,
                &current_sort_order,
                true,
                cx,
            ))
            .child(self.render_sortable_header(
                "Size",
                SortColumn::Size,
                &current_sort_column,
                &current_sort_order,
                false,
                cx,
            ))
            .child(self.render_sortable_header(
                "Modified",
                SortColumn::Modified,
                &current_sort_column,
                &current_sort_order,
                false,
                cx,
            ))
    }

    fn render_sortable_header(
        &self,
        label: &'static str,
        column: SortColumn,
        current_column: &SortColumn,
        current_order: &SortOrder,
        is_flex: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_active = std::mem::discriminant(&column) == std::mem::discriminant(current_column);
        let sort_icon = if is_active {
            match current_order {
                SortOrder::Ascending => Some(IconName::ArrowUp),
                SortOrder::Descending => Some(IconName::ArrowDown),
            }
        } else {
            None
        };

        let column_clone = column.clone();
        let width = match column {
            SortColumn::Name => None,
            SortColumn::Size => Some(px(80.)),
            SortColumn::Modified => Some(px(130.)),
            SortColumn::Extension => Some(px(80.)),
        };

        div()
            .id(ElementId::Name(format!("header-{}", label).into()))
            .flex()
            .items_center()
            .gap_1()
            .cursor_pointer()
            .hover(|s| s.text_color(cx.theme().foreground))
            .when(is_active, |el| el.text_color(cx.theme().foreground))
            .when(is_flex, |el| el.flex_1())
            .when_some(width, |el, w| el.w(w).justify_end())
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.toggle_sort(column_clone.clone(), cx);
            }))
            .child(label)
            .when_some(sort_icon, |el, icon| {
                el.child(
                    div()
                        .size_3()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon),
                )
            })
    }

    fn toggle_sort(&mut self, column: SortColumn, cx: &mut Context<Self>) {
        if std::mem::discriminant(&self.sort_column) == std::mem::discriminant(&column) {
            // Toggle order if same column
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            // New column, start with ascending
            self.sort_column = column;
            self.sort_order = SortOrder::Ascending;
        }

        // Re-sort the entries
        sort_entries(&mut self.entries, &self.sort_column, &self.sort_order);
        self.selected_indices.clear();
        cx.notify();
    }

    fn render_file_row(
        &self,
        index: usize,
        entry: &FileEntry,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.selected_indices.contains(&index);
        let entry_path = entry.path.clone();
        let is_dir = entry.is_directory();
        let is_hidden = entry.is_hidden;
        let is_directory = entry.is_directory();
        let is_parent_entry = entry.name == "..";
        let name = entry.name.clone();
        let icon_name = entry.icon_name();
        let size = entry.formatted_size();
        let date = entry.formatted_date();

        let bg_color = if is_selected {
            cx.theme().primary.opacity(0.15)
        } else {
            cx.theme().transparent
        };

        let hover_color = cx.theme().secondary;
        let icon_color = if is_directory {
            cx.theme().warning
        } else {
            cx.theme().muted_foreground
        };
        let muted_fg = cx.theme().muted_foreground;

        let entity = cx.entity().clone();

        h_flex()
            .id(ElementId::Name(format!("file-{}", index).into()))
            .h_7()
            .px_2()
            .bg(bg_color)
            .hover(|s| s.bg(hover_color))
            .border_b_1()
            .border_color(cx.theme().border.opacity(0.3))
            .items_center()
            .cursor_pointer()
            .on_click(cx.listener(move |this, event: &ClickEvent, _window, cx| {
                this.handle_click(index, event.modifiers(), cx);
            }))
            .on_double_click(cx.listener(move |this, _, _window, cx| {
                if is_dir {
                    this.navigate_to(entry_path.clone(), cx);
                } else {
                    let _ = open::that(&entry_path);
                }
            }))
            .context_menu(move |menu, _window, _cx| {
                let entity = entity.clone();

                // Don't show destructive options for parent directory entry
                if is_parent_entry {
                    return menu
                        .item(
                            gpui_component::menu::PopupMenuItem::new("Open")
                                .icon(IconName::FolderOpen)
                                .on_click({
                                    let entity = entity.clone();
                                    move |_, _, cx| {
                                        entity.update(cx, |panel, cx| {
                                            panel.open_selected(cx);
                                        });
                                    }
                                }),
                        )
                        .separator()
                        .item(
                            gpui_component::menu::PopupMenuItem::new("Refresh")
                                .icon(IconName::Loader)
                                .on_click({
                                    let entity = entity.clone();
                                    move |_, _, cx| {
                                        entity.update(cx, |panel, cx| {
                                            panel.refresh(cx);
                                        });
                                    }
                                }),
                        );
                }

                menu.item(
                    gpui_component::menu::PopupMenuItem::new("Open")
                        .icon(IconName::FolderOpen)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |panel, cx| {
                                    panel.open_selected(cx);
                                });
                            }
                        }),
                )
                .separator()
                .item(
                    gpui_component::menu::PopupMenuItem::new("Copy")
                        .icon(IconName::Copy)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |_, cx| {
                                    cx.emit(FilePanelEvent::RequestCopy);
                                });
                            }
                        }),
                )
                .item(
                    gpui_component::menu::PopupMenuItem::new("Cut")
                        .icon(IconName::Close) // No scissors, use close as placeholder
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |_, cx| {
                                    cx.emit(FilePanelEvent::RequestCut);
                                });
                            }
                        }),
                )
                .item(
                    gpui_component::menu::PopupMenuItem::new("Paste")
                        .icon(IconName::File) // Use file icon for paste
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |_, cx| {
                                    cx.emit(FilePanelEvent::RequestPaste);
                                });
                            }
                        }),
                )
                .separator()
                .item(
                    gpui_component::menu::PopupMenuItem::new("Delete")
                        .icon(IconName::Delete)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |_, cx| {
                                    cx.emit(FilePanelEvent::RequestDelete);
                                });
                            }
                        }),
                )
                .item(
                    gpui_component::menu::PopupMenuItem::new("Rename")
                        .icon(IconName::Frame) // Use frame for rename
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |_, cx| {
                                    cx.emit(FilePanelEvent::RequestRename);
                                });
                            }
                        }),
                )
                .separator()
                .item(
                    gpui_component::menu::PopupMenuItem::new("New Folder")
                        .icon(IconName::Folder)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |_, cx| {
                                    cx.emit(FilePanelEvent::RequestNewFolder);
                                });
                            }
                        }),
                )
                .item(
                    gpui_component::menu::PopupMenuItem::new("New File")
                        .icon(IconName::File)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |_, cx| {
                                    cx.emit(FilePanelEvent::RequestNewFile);
                                });
                            }
                        }),
                )
                .separator()
                .item(
                    gpui_component::menu::PopupMenuItem::new("Refresh")
                        .icon(IconName::Loader)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |panel, cx| {
                                    panel.refresh(cx);
                                });
                            }
                        }),
                )
            })
            .child(
                div()
                    .w(px(24.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(icon_color)
                    .child(icon_name),
            )
            .child(
                div()
                    .flex_1()
                    .truncate()
                    .text_sm()
                    .when(is_directory, |el: Div| el.font_weight(FontWeight::MEDIUM))
                    .when(is_hidden, |el: Div| el.text_color(muted_fg))
                    .child(name),
            )
            .child(
                div()
                    .w(px(80.))
                    .text_right()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(size),
            )
            .child(
                div()
                    .w(px(130.))
                    .text_right()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(date),
            )
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let total_files = self.entries.len().saturating_sub(1);
        let selected_count = self.selected_indices.len();
        let selected_size: u64 = self
            .selected_indices
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .map(|e| e.size)
            .sum();

        let status_text = if selected_count > 0 {
            format!(
                "{} of {} items selected ({})",
                selected_count,
                total_files,
                humansize::format_size(selected_size, humansize::BINARY)
            )
        } else {
            format!("{} items", total_files)
        };

        h_flex()
            .h_6()
            .px_2()
            .bg(cx.theme().secondary)
            .border_t_1()
            .border_color(cx.theme().border)
            .items_center()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(status_text)
    }
}

impl EventEmitter<FilePanelEvent> for FilePanel {}

impl Focusable for FilePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FilePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entries: Vec<_> = self.entries.iter().cloned().enumerate().collect();

        // Use a subtle highlight for active panel
        let active_indicator_color = if self.is_active {
            cx.theme().primary.opacity(0.1)
        } else {
            cx.theme().transparent
        };

        v_flex()
            .id(self.id)
            .flex_1()
            .size_full()
            .bg(active_indicator_color)
            .overflow_hidden()
            .key_context("FilePanel")
            .on_action(cx.listener(Self::handle_move_up))
            .on_action(cx.listener(Self::handle_move_down))
            .on_action(cx.listener(Self::handle_move_to_start))
            .on_action(cx.listener(Self::handle_move_to_end))
            .on_action(cx.listener(Self::handle_page_up))
            .on_action(cx.listener(Self::handle_page_down))
            .track_focus(&self.focus_handle)
            .on_click(cx.listener(|this, _, window, cx| {
                cx.emit(FilePanelEvent::PanelFocused);
                this.focus_handle.focus(window);
            }))
            .child(self.render_navigation_bar(cx))
            .child(self.render_header(cx))
            .child(
                v_flex().flex_1().size_full().overflow_hidden().child(
                    div().size_full().overflow_y_scrollbar().children(
                        entries
                            .iter()
                            .map(|(idx, entry)| self.render_file_row(*idx, entry, cx)),
                    ),
                ),
            )
            .child(self.render_status_bar(cx))
    }
}
