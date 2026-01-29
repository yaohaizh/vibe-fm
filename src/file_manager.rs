use crate::drive_selector::{DriveSelector, DriveSelectorEvent, PanelSide};
use crate::file_entry::FileEntry;
use crate::file_ops;
use crate::file_panel::{FilePanel, FilePanelEvent};
use crate::filter_bar::{FilterBar, FilterBarEvent};
use crate::function_bar::{FunctionBar, FunctionBarAction, FunctionBarEvent};
use crate::status_bar::{ActivePanel, StatusBar};
use crate::toolbar::{Toolbar, ToolbarAction, ToolbarEvent};
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme};
use std::path::PathBuf;

// Define actions for keyboard shortcuts
actions!(
    file_manager,
    [
        Copy,
        Cut,
        Paste,
        Delete,
        Refresh,
        NewFolder,
        NewFile,
        SelectAll,
        SwitchPanel,
        OpenSelected,
        NavigateUp,
        NavigateBack,
        NavigateForward,
        ToggleHidden,
        FocusPath,
        // Function key actions
        View,
        Edit,
        Move,
        Rename,
        Exit,
        // Filter
        ShowFilter,
        ClearFilter,
    ]
);

/// Register global key bindings for file manager actions
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-c", Copy, Some("FileManager")),
        KeyBinding::new("ctrl-x", Cut, Some("FileManager")),
        KeyBinding::new("ctrl-v", Paste, Some("FileManager")),
        KeyBinding::new("delete", Delete, Some("FileManager")),
        KeyBinding::new("f5", Refresh, Some("FileManager")),
        KeyBinding::new("ctrl-shift-n", NewFolder, Some("FileManager")),
        KeyBinding::new("ctrl-n", NewFile, Some("FileManager")),
        KeyBinding::new("ctrl-a", SelectAll, Some("FileManager")),
        KeyBinding::new("tab", SwitchPanel, Some("FileManager")),
        KeyBinding::new("enter", OpenSelected, Some("FileManager")),
        KeyBinding::new("backspace", NavigateUp, Some("FileManager")),
        KeyBinding::new("alt-left", NavigateBack, Some("FileManager")),
        KeyBinding::new("alt-right", NavigateForward, Some("FileManager")),
        KeyBinding::new("ctrl-h", ToggleHidden, Some("FileManager")),
        // Function keys
        KeyBinding::new("f3", View, Some("FileManager")),
        KeyBinding::new("f4", Edit, Some("FileManager")),
        KeyBinding::new("f6", Move, Some("FileManager")),
        KeyBinding::new("f7", NewFolder, Some("FileManager")),
        KeyBinding::new("f8", Delete, Some("FileManager")),
        KeyBinding::new("f2", Rename, Some("FileManager")),
        KeyBinding::new("f9", Rename, Some("FileManager")),
        KeyBinding::new("f10", Exit, Some("FileManager")),
        KeyBinding::new("alt-f4", Exit, Some("FileManager")),
        // Filter
        KeyBinding::new("ctrl-f", ShowFilter, Some("FileManager")),
        KeyBinding::new("escape", ClearFilter, Some("FileManager")),
    ]);
}

pub struct FileManager {
    left_panel: Entity<FilePanel>,
    right_panel: Entity<FilePanel>,
    toolbar: Entity<Toolbar>,
    status_bar: Entity<StatusBar>,
    function_bar: Entity<FunctionBar>,
    filter_bar: Entity<FilterBar>,
    left_drive_selector: Entity<DriveSelector>,
    right_drive_selector: Entity<DriveSelector>,
    active_panel: ActivePanel,
    clipboard: Option<Clipboard>,
    focus_handle: FocusHandle,
}

#[derive(Clone)]
struct Clipboard {
    entries: Vec<FileEntry>,
    is_cut: bool,
}

impl FileManager {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        #[cfg(target_os = "windows")]
        let left_path = PathBuf::from("C:\\");
        #[cfg(not(target_os = "windows"))]
        let left_path = home_dir.clone();

        let right_path = home_dir;

        let left_panel = cx.new(|cx| FilePanel::new("left-panel", left_path, window, cx));
        let right_panel = cx.new(|cx| FilePanel::new("right-panel", right_path, window, cx));
        let toolbar = cx.new(|_cx| Toolbar::new());
        let status_bar = cx.new(|_cx| StatusBar::new());
        let function_bar = cx.new(|_cx| FunctionBar::new());
        let filter_bar = cx.new(|cx| FilterBar::new(cx));
        let left_drive_selector = cx.new(|_cx| DriveSelector::new(PanelSide::Left));
        let right_drive_selector = cx.new(|_cx| DriveSelector::new(PanelSide::Right));

        cx.subscribe(&left_panel, Self::on_left_panel_event)
            .detach();
        cx.subscribe(&right_panel, Self::on_right_panel_event)
            .detach();
        cx.subscribe(&toolbar, Self::on_toolbar_event).detach();
        cx.subscribe(&function_bar, Self::on_function_bar_event)
            .detach();
        cx.subscribe(&filter_bar, Self::on_filter_bar_event)
            .detach();
        cx.subscribe(&left_drive_selector, Self::on_drive_selected)
            .detach();
        cx.subscribe(&right_drive_selector, Self::on_drive_selected)
            .detach();

        left_panel.update(cx, |panel, cx| {
            panel.set_active(true, cx);
        });

        let focus_handle = cx.focus_handle();

        Self {
            left_panel,
            right_panel,
            toolbar,
            status_bar,
            function_bar,
            filter_bar,
            left_drive_selector,
            right_drive_selector,
            active_panel: ActivePanel::Left,
            clipboard: None,
            focus_handle,
        }
    }

    // Action handlers for keyboard shortcuts
    fn handle_copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        self.copy_selected(cx);
    }

    fn handle_cut(&mut self, _: &Cut, _window: &mut Window, cx: &mut Context<Self>) {
        self.cut_selected(cx);
    }

    fn handle_paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        self.paste(cx);
    }

    fn handle_delete(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        self.delete_selected(cx);
    }

    fn handle_refresh(&mut self, _: &Refresh, _window: &mut Window, cx: &mut Context<Self>) {
        self.refresh_panels(cx);
    }

    fn handle_new_folder(&mut self, _: &NewFolder, _window: &mut Window, cx: &mut Context<Self>) {
        self.create_folder(cx);
    }

    fn handle_new_file(&mut self, _: &NewFile, _window: &mut Window, cx: &mut Context<Self>) {
        self.create_file(cx);
    }

    fn handle_select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.active_panel_entity().update(cx, |panel, cx| {
            panel.select_all(cx);
        });
    }

    fn handle_switch_panel(
        &mut self,
        _: &SwitchPanel,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_panel = match self.active_panel {
            ActivePanel::Left => ActivePanel::Right,
            ActivePanel::Right => ActivePanel::Left,
        };
        self.set_active_panel(new_panel, cx);
    }

    fn handle_open_selected(
        &mut self,
        _: &OpenSelected,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_panel_entity().update(cx, |panel, cx| {
            panel.open_selected(cx);
        });
    }

    fn handle_navigate_up(&mut self, _: &NavigateUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.active_panel_entity().update(cx, |panel, cx| {
            panel.navigate_up(cx);
        });
    }

    fn handle_navigate_back(
        &mut self,
        _: &NavigateBack,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_panel_entity().update(cx, |panel, cx| {
            panel.navigate_back(cx);
        });
    }

    fn handle_navigate_forward(
        &mut self,
        _: &NavigateForward,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_panel_entity().update(cx, |panel, cx| {
            panel.navigate_forward(cx);
        });
    }

    fn handle_toggle_hidden(
        &mut self,
        _: &ToggleHidden,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_hidden(cx);
    }

    fn handle_view(&mut self, _: &View, _window: &mut Window, cx: &mut Context<Self>) {
        self.view_selected(cx);
    }

    fn handle_edit(&mut self, _: &Edit, _window: &mut Window, cx: &mut Context<Self>) {
        self.edit_selected(cx);
    }

    fn handle_move(&mut self, _: &Move, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_selected(cx);
    }

    fn handle_rename(&mut self, _: &Rename, _window: &mut Window, cx: &mut Context<Self>) {
        self.rename_selected(cx);
    }

    fn handle_exit(&mut self, _: &Exit, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn handle_show_filter(&mut self, _: &ShowFilter, window: &mut Window, cx: &mut Context<Self>) {
        self.filter_bar.update(cx, |bar, cx| {
            bar.show(window, cx);
        });
    }

    fn handle_clear_filter(
        &mut self,
        _: &ClearFilter,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.filter_bar.update(cx, |bar, cx| {
            bar.hide(cx);
        });
        // Clear filter on active panel
        self.active_panel_entity().update(cx, |panel, cx| {
            panel.clear_filter(cx);
        });
    }

    fn on_left_panel_event(
        &mut self,
        _panel: Entity<FilePanel>,
        event: &FilePanelEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            FilePanelEvent::PanelFocused => {
                self.set_active_panel(ActivePanel::Left, cx);
            }
            FilePanelEvent::SelectionChanged(entries) => {
                self.status_bar.update(cx, |bar, cx| {
                    bar.update_left(String::new(), entries.len(), 0, cx);
                });
            }
            FilePanelEvent::PathChanged(_path) => {}
            FilePanelEvent::FileOpened(path) => {
                let _ = open::that(path);
            }
            FilePanelEvent::DirectoryEntered(_path) => {}
            // Context menu events
            FilePanelEvent::RequestCopy => self.copy_selected(cx),
            FilePanelEvent::RequestCut => self.cut_selected(cx),
            FilePanelEvent::RequestPaste => self.paste(cx),
            FilePanelEvent::RequestDelete => self.delete_selected(cx),
            FilePanelEvent::RequestNewFolder => self.create_folder(cx),
            FilePanelEvent::RequestNewFile => self.create_file(cx),
            FilePanelEvent::RequestRename => self.rename_selected(cx),
            FilePanelEvent::RequestRefresh => self.refresh_panels(cx),
        }
    }

    fn on_right_panel_event(
        &mut self,
        _panel: Entity<FilePanel>,
        event: &FilePanelEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            FilePanelEvent::PanelFocused => {
                self.set_active_panel(ActivePanel::Right, cx);
            }
            FilePanelEvent::SelectionChanged(entries) => {
                self.status_bar.update(cx, |bar, cx| {
                    bar.update_right(String::new(), entries.len(), 0, cx);
                });
            }
            FilePanelEvent::PathChanged(_) => {}
            FilePanelEvent::FileOpened(path) => {
                let _ = open::that(path);
            }
            FilePanelEvent::DirectoryEntered(_) => {}
            // Context menu events
            FilePanelEvent::RequestCopy => self.copy_selected(cx),
            FilePanelEvent::RequestCut => self.cut_selected(cx),
            FilePanelEvent::RequestPaste => self.paste(cx),
            FilePanelEvent::RequestDelete => self.delete_selected(cx),
            FilePanelEvent::RequestNewFolder => self.create_folder(cx),
            FilePanelEvent::RequestNewFile => self.create_file(cx),
            FilePanelEvent::RequestRename => self.rename_selected(cx),
            FilePanelEvent::RequestRefresh => self.refresh_panels(cx),
        }
    }

    fn on_toolbar_event(
        &mut self,
        _toolbar: Entity<Toolbar>,
        event: &ToolbarEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ToolbarEvent::Action(action) => match action {
                ToolbarAction::Copy => self.copy_selected(cx),
                ToolbarAction::Cut => self.cut_selected(cx),
                ToolbarAction::Paste => self.paste(cx),
                ToolbarAction::Delete => self.delete_selected(cx),
                ToolbarAction::NewFolder => self.create_folder(cx),
                ToolbarAction::NewFile => self.create_file(cx),
                ToolbarAction::Rename => self.rename_selected(cx),
                ToolbarAction::Refresh => self.refresh_panels(cx),
                ToolbarAction::ToggleHidden => self.toggle_hidden(cx),
                ToolbarAction::SwapPanels => self.swap_panels(cx),
                ToolbarAction::Search => {}
                ToolbarAction::Settings => {}
            },
        }
    }

    fn on_drive_selected(
        &mut self,
        _selector: Entity<DriveSelector>,
        event: &DriveSelectorEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            DriveSelectorEvent::DriveSelected(path, panel_side) => {
                let panel = match panel_side {
                    PanelSide::Left => &self.left_panel,
                    PanelSide::Right => &self.right_panel,
                };
                panel.update(cx, |panel, cx| {
                    panel.navigate_to(path.clone(), cx);
                });
            }
        }
    }

    fn on_function_bar_event(
        &mut self,
        _bar: Entity<FunctionBar>,
        event: &FunctionBarEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            FunctionBarEvent::Action(action) => match action {
                FunctionBarAction::View => self.view_selected(cx),
                FunctionBarAction::Edit => self.edit_selected(cx),
                FunctionBarAction::Copy => self.copy_to_other_panel(cx),
                FunctionBarAction::Move => self.move_selected(cx),
                FunctionBarAction::NewFolder => self.create_folder(cx),
                FunctionBarAction::Delete => self.delete_selected(cx),
                FunctionBarAction::Rename => self.rename_selected(cx),
                FunctionBarAction::Exit => cx.quit(),
            },
        }
    }

    fn on_filter_bar_event(
        &mut self,
        _bar: Entity<FilterBar>,
        event: &FilterBarEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            FilterBarEvent::FilterChanged(text) => {
                self.active_panel_entity().update(cx, |panel, cx| {
                    panel.set_filter(text.clone(), cx);
                });
            }
            FilterBarEvent::FilterCleared => {
                self.active_panel_entity().update(cx, |panel, cx| {
                    panel.clear_filter(cx);
                });
            }
            FilterBarEvent::Dismissed => {
                // Filter bar was closed
            }
        }
    }

    fn set_active_panel(&mut self, panel: ActivePanel, cx: &mut Context<Self>) {
        self.active_panel = panel;

        let (active, inactive) = match panel {
            ActivePanel::Left => (&self.left_panel, &self.right_panel),
            ActivePanel::Right => (&self.right_panel, &self.left_panel),
        };

        active.update(cx, |p, cx| p.set_active(true, cx));
        inactive.update(cx, |p, cx| p.set_active(false, cx));

        self.status_bar.update(cx, |bar, cx| {
            bar.set_active_panel(panel, cx);
        });
    }

    fn active_panel_entity(&self) -> &Entity<FilePanel> {
        match self.active_panel {
            ActivePanel::Left => &self.left_panel,
            ActivePanel::Right => &self.right_panel,
        }
    }

    fn copy_selected(&mut self, cx: &mut Context<Self>) {
        let entries = self.active_panel_entity().read(cx).selected_entries();
        if !entries.is_empty() {
            self.clipboard = Some(Clipboard {
                entries,
                is_cut: false,
            });
        }
    }

    fn cut_selected(&mut self, cx: &mut Context<Self>) {
        let entries = self.active_panel_entity().read(cx).selected_entries();
        if !entries.is_empty() {
            self.clipboard = Some(Clipboard {
                entries,
                is_cut: true,
            });
        }
    }

    fn paste(&mut self, cx: &mut Context<Self>) {
        if let Some(clipboard) = self.clipboard.take() {
            let target_path = self.active_panel_entity().read(cx).current_path().clone();

            for entry in &clipboard.entries {
                let source = &entry.path;
                let dest = target_path.join(&entry.name);

                if clipboard.is_cut {
                    if let Err(e) = std::fs::rename(source, &dest) {
                        log::error!("Failed to move {:?}: {}", source, e);
                    }
                } else {
                    if entry.is_directory() {
                        if let Err(e) = file_ops::copy_dir_recursive(&entry.path, &dest) {
                            log::error!("Failed to copy directory {:?}: {}", entry.path, e);
                        }
                    } else {
                        if let Err(e) = std::fs::copy(&entry.path, &dest) {
                            log::error!("Failed to copy {:?}: {}", entry.path, e);
                        }
                    }
                }
            }

            self.refresh_panels(cx);

            if !clipboard.is_cut {
                self.clipboard = Some(clipboard);
            }
        }
    }

    fn delete_selected(&mut self, cx: &mut Context<Self>) {
        let entries = self.active_panel_entity().read(cx).selected_entries();

        for entry in entries {
            if entry.name == ".." {
                continue;
            }

            let result = if entry.is_directory() {
                std::fs::remove_dir_all(&entry.path)
            } else {
                std::fs::remove_file(&entry.path)
            };

            if let Err(e) = result {
                log::error!("Failed to delete {:?}: {}", entry.path, e);
            }
        }

        self.refresh_panels(cx);
    }

    fn create_folder(&mut self, cx: &mut Context<Self>) {
        let current_path = self.active_panel_entity().read(cx).current_path().clone();
        let new_folder = current_path.join("New Folder");

        if let Err(e) = std::fs::create_dir(&new_folder) {
            log::error!("Failed to create folder: {}", e);
        }

        self.active_panel_entity().update(cx, |panel, cx| {
            panel.refresh(cx);
        });
    }

    fn create_file(&mut self, cx: &mut Context<Self>) {
        let current_path = self.active_panel_entity().read(cx).current_path().clone();
        let new_file = current_path.join("New File.txt");

        if let Err(e) = std::fs::File::create(&new_file) {
            log::error!("Failed to create file: {}", e);
        }

        self.active_panel_entity().update(cx, |panel, cx| {
            panel.refresh(cx);
        });
    }

    fn rename_selected(&mut self, _cx: &mut Context<Self>) {
        // TODO: Implement rename dialog
    }

    fn refresh_panels(&mut self, cx: &mut Context<Self>) {
        self.left_panel.update(cx, |panel, cx| {
            panel.refresh(cx);
        });
        self.right_panel.update(cx, |panel, cx| {
            panel.refresh(cx);
        });
    }

    fn toggle_hidden(&mut self, cx: &mut Context<Self>) {
        self.active_panel_entity().update(cx, |panel, cx| {
            panel.toggle_hidden(cx);
        });
    }

    fn swap_panels(&mut self, cx: &mut Context<Self>) {
        let left_path = self.left_panel.read(cx).current_path().clone();
        let right_path = self.right_panel.read(cx).current_path().clone();

        self.left_panel.update(cx, |panel, cx| {
            panel.navigate_to(right_path, cx);
        });
        self.right_panel.update(cx, |panel, cx| {
            panel.navigate_to(left_path, cx);
        });
    }

    fn inactive_panel_entity(&self) -> &Entity<FilePanel> {
        match self.active_panel {
            ActivePanel::Left => &self.right_panel,
            ActivePanel::Right => &self.left_panel,
        }
    }

    fn view_selected(&mut self, cx: &mut Context<Self>) {
        let entries = self.active_panel_entity().read(cx).selected_entries();
        if let Some(entry) = entries.first() {
            if !entry.is_directory() {
                let _ = open::that(&entry.path);
            }
        }
    }

    fn edit_selected(&mut self, cx: &mut Context<Self>) {
        // For now, edit behaves the same as view - opens with default application
        // In the future, this could open with a specific editor
        self.view_selected(cx);
    }

    fn move_selected(&mut self, cx: &mut Context<Self>) {
        let entries = self.active_panel_entity().read(cx).selected_entries();
        if entries.is_empty() {
            return;
        }

        let target_path = self.inactive_panel_entity().read(cx).current_path().clone();

        for entry in &entries {
            if entry.name == ".." {
                continue;
            }
            let dest = target_path.join(&entry.name);
            if let Err(e) = std::fs::rename(&entry.path, &dest) {
                log::error!("Failed to move {:?}: {}", entry.path, e);
            }
        }

        self.refresh_panels(cx);
    }

    fn copy_to_other_panel(&mut self, cx: &mut Context<Self>) {
        let entries = self.active_panel_entity().read(cx).selected_entries();
        if entries.is_empty() {
            return;
        }

        let target_path = self.inactive_panel_entity().read(cx).current_path().clone();

        for entry in &entries {
            if entry.name == ".." {
                continue;
            }
            let dest = target_path.join(&entry.name);

            if entry.is_directory() {
                if let Err(e) = file_ops::copy_dir_recursive(&entry.path, &dest) {
                    log::error!("Failed to copy directory {:?}: {}", entry.path, e);
                }
            } else {
                if let Err(e) = std::fs::copy(&entry.path, &dest) {
                    log::error!("Failed to copy {:?}: {}", entry.path, e);
                }
            }
        }

        self.refresh_panels(cx);
    }
}

impl Render for FileManager {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let left_border_color = if self.active_panel == ActivePanel::Left {
            cx.theme().primary
        } else {
            cx.theme().border
        };

        let right_border_color = if self.active_panel == ActivePanel::Right {
            cx.theme().primary
        } else {
            cx.theme().border
        };

        v_flex()
            .id("file-manager")
            .size_full()
            .key_context("FileManager")
            .on_action(cx.listener(Self::handle_copy))
            .on_action(cx.listener(Self::handle_cut))
            .on_action(cx.listener(Self::handle_paste))
            .on_action(cx.listener(Self::handle_delete))
            .on_action(cx.listener(Self::handle_refresh))
            .on_action(cx.listener(Self::handle_new_folder))
            .on_action(cx.listener(Self::handle_new_file))
            .on_action(cx.listener(Self::handle_select_all))
            .on_action(cx.listener(Self::handle_switch_panel))
            .on_action(cx.listener(Self::handle_open_selected))
            .on_action(cx.listener(Self::handle_navigate_up))
            .on_action(cx.listener(Self::handle_navigate_back))
            .on_action(cx.listener(Self::handle_navigate_forward))
            .on_action(cx.listener(Self::handle_toggle_hidden))
            .on_action(cx.listener(Self::handle_view))
            .on_action(cx.listener(Self::handle_edit))
            .on_action(cx.listener(Self::handle_move))
            .on_action(cx.listener(Self::handle_rename))
            .on_action(cx.listener(Self::handle_exit))
            .on_action(cx.listener(Self::handle_show_filter))
            .on_action(cx.listener(Self::handle_clear_filter))
            .track_focus(&self.focus_handle)
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            // Toolbar at top
            .child(self.toolbar.clone())
            // Filter bar (shown when active)
            .child(self.filter_bar.clone())
            // Main content area with dual panels
            .child(
                h_flex()
                    .flex_1()
                    .gap_2()
                    .p_2()
                    .overflow_hidden()
                    // Left panel container (drive selector + file panel)
                    .child(
                        v_flex()
                            .flex_1()
                            .size_full()
                            .overflow_hidden()
                            .border_1()
                            .border_color(left_border_color)
                            .rounded_md()
                            .bg(cx.theme().background)
                            .child(self.left_drive_selector.clone())
                            .child(self.left_panel.clone()),
                    )
                    // Right panel container (drive selector + file panel)
                    .child(
                        v_flex()
                            .flex_1()
                            .size_full()
                            .overflow_hidden()
                            .border_1()
                            .border_color(right_border_color)
                            .rounded_md()
                            .bg(cx.theme().background)
                            .child(self.right_drive_selector.clone())
                            .child(self.right_panel.clone()),
                    ),
            )
            // Status bar at bottom
            .child(self.status_bar.clone())
            // Function bar at the very bottom
            .child(self.function_bar.clone())
    }
}

impl Focusable for FileManager {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
