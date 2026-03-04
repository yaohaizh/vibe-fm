use crate::file_entry::{sort_entries, FileEntry, SortColumn, SortOrder};
use crate::settings::{DateFormat, SizeFormat};
use crate::shell_context_menu;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::breadcrumb::{Breadcrumb, BreadcrumbItem};
use gpui_component::button::{Button, ButtonVariants};
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
    filtered_entries: Vec<FileEntry>,
    filter_text: String,
    selected_indices: Vec<usize>,
    last_selected_index: Option<usize>,
    sort_column: SortColumn,
    sort_order: SortOrder,
    show_hidden: bool,
    history: Vec<PathBuf>,
    history_index: usize,
    is_active: bool,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    // Display format settings
    date_format: DateFormat,
    size_format: SizeFormat,
    show_file_extensions: bool,
    single_click_to_open: bool,
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
        let filtered_entries = entries.clone();
        let focus_handle = cx.focus_handle();

        Self {
            id,
            current_path: initial_path.clone(),
            entries,
            filtered_entries,
            filter_text: String::new(),
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
            // Default display settings
            date_format: DateFormat::default(),
            size_format: SizeFormat::default(),
            show_file_extensions: true,
            single_click_to_open: false,
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
        self.apply_filter();
        self.selected_indices.clear();
        self.last_selected_index = None;
        cx.notify();
    }

    pub fn set_filter(&mut self, filter: String, cx: &mut Context<Self>) {
        self.filter_text = filter;
        self.apply_filter();
        self.selected_indices.clear();
        self.last_selected_index = None;
        cx.notify();
    }

    pub fn clear_filter(&mut self, cx: &mut Context<Self>) {
        self.filter_text.clear();
        self.apply_filter();
        cx.notify();
    }

    fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_entries = self.entries.clone();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_entries = self
                .entries
                .iter()
                .filter(|e| e.name == ".." || e.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect();
        }
    }

    pub fn toggle_hidden(&mut self, cx: &mut Context<Self>) {
        self.show_hidden = !self.show_hidden;
        self.refresh(cx);
    }

    pub fn set_show_hidden(&mut self, show: bool, cx: &mut Context<Self>) {
        if self.show_hidden != show {
            self.show_hidden = show;
            self.refresh(cx);
        }
    }

    /// Update display format settings
    pub fn set_date_format(&mut self, format: DateFormat, cx: &mut Context<Self>) {
        self.date_format = format;
        cx.notify();
    }

    pub fn set_size_format(&mut self, format: SizeFormat, cx: &mut Context<Self>) {
        self.size_format = format;
        cx.notify();
    }

    pub fn set_show_file_extensions(&mut self, show: bool, cx: &mut Context<Self>) {
        self.show_file_extensions = show;
        cx.notify();
    }

    pub fn set_single_click_to_open(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.single_click_to_open = enabled;
        cx.notify();
    }

    /// Set the sort column and order
    pub fn set_sort(&mut self, column: SortColumn, order: SortOrder, cx: &mut Context<Self>) {
        self.sort_column = column;
        self.sort_order = order;
        sort_entries(&mut self.entries, &self.sort_column, &self.sort_order);
        self.apply_filter();
        self.selected_indices.clear();
        cx.notify();
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
            .filter_map(|&i| self.filtered_entries.get(i).cloned())
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
            .filtered_entries
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
            if let Some(entry) = self.filtered_entries.get(index) {
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
        let max_index = self.filtered_entries.len().saturating_sub(1);
        let new_index = if current < max_index {
            current + 1
        } else {
            max_index
        };
        self.select_single(new_index, cx);
    }

    pub fn move_selection_to_start(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_entries.is_empty() {
            self.select_single(0, cx);
        }
    }

    pub fn move_selection_to_end(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_entries.is_empty() {
            self.select_single(self.filtered_entries.len() - 1, cx);
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
        let max_index = self.filtered_entries.len().saturating_sub(1);
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
            .p_2()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
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
            .h_8()
            .px_3()
            .bg(cx.theme().background)
            .border_b_1()
            .border_color(cx.theme().border)
            .items_center()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(cx.theme().foreground.opacity(0.7))
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
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .hover(|s| {
                s.bg(cx.theme().secondary.opacity(0.5))
                    .text_color(cx.theme().foreground)
            })
            .when(is_active, |el| {
                el.text_color(cx.theme().primary)
                    .font_weight(FontWeight::BOLD)
            })
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
        self.apply_filter();
        self.selected_indices.clear();
        cx.notify();
    }

    fn render_file_row(
        &self,
        index: usize,
        entry: &FileEntry,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.selected_indices.contains(&index);
        let entry_path = entry.path.clone();
        let is_dir = entry.is_directory();
        let is_hidden = entry.is_hidden;
        let is_directory = entry.is_directory();
        let name = entry.display_name(self.show_file_extensions);
        let icon_name = entry.icon_name();
        let size = entry.formatted_size_with_format(self.size_format);
        let date = entry.formatted_date_with_format(self.date_format);
        let single_click_open = self.single_click_to_open;

        let bg_color = if is_selected {
            cx.theme().primary.opacity(0.2)
        } else {
            cx.theme().transparent
        };

        let hover_color = if is_selected {
            cx.theme().primary.opacity(0.25)
        } else {
            cx.theme().secondary.opacity(0.5)
        };

        let icon_color = if is_directory {
            cx.theme().warning
        } else {
            cx.theme().muted_foreground
        };
        let muted_fg = cx.theme().muted_foreground;

        let entry_path_for_click = entry_path.clone();
        let entry_path_for_context = entry_path.clone();

        h_flex()
            .id(ElementId::Name(format!("file-{}", index).into()))
            .h_7()
            .px_2()
            .mx_1()
            .my_px()
            .bg(bg_color)
            .hover(|s| s.bg(hover_color))
            .rounded_md()
            .items_center()
            .cursor_pointer()
            .on_click(cx.listener(move |this, event: &ClickEvent, _window, cx| {
                this.handle_click(index, event.modifiers(), cx);
                // Single click to open if enabled (and no modifiers pressed)
                if single_click_open && !event.modifiers().control && !event.modifiers().shift {
                    if is_dir {
                        this.navigate_to(entry_path_for_click.clone(), cx);
                    } else {
                        let _ = open::that(&entry_path_for_click);
                    }
                }
            }))
            .on_double_click(cx.listener(move |this, _, _window, cx| {
                if is_dir {
                    this.navigate_to(entry_path.clone(), cx);
                } else {
                    let _ = open::that(&entry_path);
                }
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, _event: &MouseDownEvent, _window, cx| {
                    // Select the item if not already selected
                    if !this.selected_indices.contains(&index) {
                        this.select_single(index, cx);
                    }

                    // Get selected paths
                    let selected_entries = this.selected_entries();
                    let paths: Vec<PathBuf> = if selected_entries.is_empty() {
                        vec![entry_path_for_context.clone()]
                    } else {
                        selected_entries.iter().map(|e| e.path.clone()).collect()
                    };

                    // Show the native context menu
                    // Run this in a separate thread to not block the UI
                    std::thread::spawn(move || {
                        let _ = shell_context_menu::show_context_menu_for_paths(&paths);
                    });

                    // Stop propagation
                    cx.stop_propagation();
                }),
            )
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
        let filtered_files = self.filtered_entries.len().saturating_sub(1);
        let selected_count = self.selected_indices.len();
        let selected_size: u64 = self
            .selected_indices
            .iter()
            .filter_map(|&i| self.filtered_entries.get(i))
            .map(|e| e.size)
            .sum();

        let status_text = if selected_count > 0 {
            format!(
                "{} of {} items selected ({})",
                selected_count,
                filtered_files,
                humansize::format_size(selected_size, humansize::BINARY)
            )
        } else if !self.filter_text.is_empty() {
            format!("{} of {} items (filtered)", filtered_files, total_files)
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entries: Vec<_> = self.filtered_entries.iter().cloned().enumerate().collect();

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
                            .map(|(idx, entry)| self.render_file_row(*idx, entry, window, cx)),
                    ),
                ),
            )
            .child(self.render_status_bar(cx))
    }
}
