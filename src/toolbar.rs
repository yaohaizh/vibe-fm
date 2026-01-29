use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, ActiveTheme, IconName, Sizable};

pub struct Toolbar {
    pub show_hidden: bool,
}

pub enum ToolbarAction {
    Copy,
    Cut,
    Paste,
    Delete,
    NewFolder,
    NewFile,
    Rename,
    Refresh,
    ToggleHidden,
    SwapPanels,
    Search,
    Settings,
    Bookmarks,
    AddBookmark,
}

pub enum ToolbarEvent {
    Action(ToolbarAction),
}

impl EventEmitter<ToolbarEvent> for Toolbar {}

impl Toolbar {
    pub fn new() -> Self {
        Self { show_hidden: true }
    }

    pub fn set_show_hidden(&mut self, show: bool, cx: &mut Context<Self>) {
        self.show_hidden = show;
        cx.notify();
    }
}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h(px(40.))
            .px_2()
            .gap_1()
            .items_center()
            .bg(cx.theme().secondary)
            .border_b_1()
            .border_color(cx.theme().border)
            // File operations group
            .child(
                h_flex()
                    .gap_0p5()
                    .child(
                        Button::new("tb-copy")
                            .icon(IconName::Copy)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("Copy (Ctrl+C)")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::Copy));
                            })),
                    )
                    .child(
                        Button::new("tb-paste")
                            .icon(IconName::Inbox)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("Paste (Ctrl+V)")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::Paste));
                            })),
                    ),
            )
            .child(div().w_px().h_6().bg(cx.theme().border))
            // Create operations
            .child(
                h_flex()
                    .gap_0p5()
                    .child(
                        Button::new("tb-new-folder")
                            .icon(IconName::FolderOpen)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("New Folder (F7)")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::NewFolder));
                            })),
                    )
                    .child(
                        Button::new("tb-new-file")
                            .icon(IconName::File)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("New File")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::NewFile));
                            })),
                    ),
            )
            .child(div().w_px().h_6().bg(cx.theme().border))
            // Edit operations
            .child(
                h_flex().gap_0p5().child(
                    Button::new("tb-delete")
                        .icon(IconName::Delete)
                        .ghost()
                        .compact()
                        .small()
                        .tooltip("Delete (Del)")
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.emit(ToolbarEvent::Action(ToolbarAction::Delete));
                        })),
                ),
            )
            .child(div().w_px().h_6().bg(cx.theme().border))
            // View operations
            .child(
                h_flex()
                    .gap_0p5()
                    .child(
                        Button::new("tb-swap")
                            .icon(IconName::PanelLeft)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("Swap Panels")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::SwapPanels));
                            })),
                    )
                    .child(
                        Button::new("tb-refresh")
                            .icon(IconName::Loader)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("Refresh (F5)")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::Refresh));
                            })),
                    ),
            )
            .child(div().flex_1())
            // Right side
            .child(
                h_flex()
                    .gap_0p5()
                    .child(
                        Button::new("tb-bookmarks")
                            .icon(IconName::Star)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("Bookmarks (Ctrl+B)")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::Bookmarks));
                            })),
                    )
                    .child(
                        Button::new("tb-add-bookmark")
                            .icon(IconName::Plus)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("Add Bookmark (Ctrl+D)")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::AddBookmark));
                            })),
                    )
                    .child(
                        Button::new("tb-search")
                            .icon(IconName::Search)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("Search (Ctrl+F)")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::Search));
                            })),
                    )
                    .child(
                        Button::new("tb-settings")
                            .icon(IconName::Settings)
                            .ghost()
                            .compact()
                            .small()
                            .tooltip("Settings")
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(ToolbarEvent::Action(ToolbarAction::Settings));
                            })),
                    ),
            )
    }
}
