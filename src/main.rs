mod drive_selector;
mod file_entry;
mod file_manager;
mod file_ops;
mod file_panel;
mod filter_bar;
mod function_bar;
mod status_bar;
mod toolbar;

use file_manager::FileManager;
use gpui::*;
use gpui_component::*;

fn main() {
    env_logger::init();

    // Initialize with bundled assets that include all icons
    let app = Application::new().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        // Register global key bindings
        file_manager::register_keybindings(cx);
        file_panel::register_keybindings(cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(100.), px(100.)),
                    size: size(px(1400.), px(900.)),
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("Vibe File Manager".into()),
                    appears_transparent: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| FileManager::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .ok();
    });
}
