use std::sync::atomic::AtomicI32;
use tauri::{AboutMetadata, CustomMenuItem, Menu, MenuItem, Submenu, WindowBuilder, WindowUrl};
#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

pub fn create_default_menu() -> Menu {
    let menu = Menu::new()
        .add_submenu(Submenu::new(
            "Socketor",
            Menu::new()
                .add_native_item(MenuItem::About("Socketor".to_string(), AboutMetadata::new()))
                .add_native_item(MenuItem::Hide)
                .add_native_item(MenuItem::HideOthers)
                .add_native_item(MenuItem::ShowAll)
                .add_native_item(MenuItem::Quit),
        ))
        .add_submenu(Submenu::new(
            "File",
            Menu::new()
                .add_item(CustomMenuItem::new("new-instance", "Open New Window").accelerator("CmdOrControl+N"))
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::CloseWindow),
        ))
        .add_submenu(Submenu::new(
            "Edit",
            Menu::new()
                .add_native_item(MenuItem::Undo)
                .add_native_item(MenuItem::Redo)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Cut)
                .add_native_item(MenuItem::Copy)
                .add_native_item(MenuItem::Paste)
                .add_native_item(MenuItem::SelectAll),
        ))
        .add_submenu(Submenu::new(
            "View",
            Menu::new()
                .add_native_item(MenuItem::EnterFullScreen),
        ))
        .add_submenu(Submenu::new(
            "Window",
            Menu::new()
                .add_native_item(MenuItem::Minimize)
                .add_native_item(MenuItem::Zoom),
        ));
    menu
}

// Increment the instance count and create a new window
static INSTANCE_COUNT: AtomicI32 = AtomicI32::new(0);

pub fn create_new_instance(app: tauri::AppHandle) {
    // Increment the instance count
    let instance_count = INSTANCE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let window_label = format!("instance_{:04}", instance_count);

    println!("Creating new window with label: {}", window_label);
    let window = WindowBuilder::new(&app, window_label, WindowUrl::App("index.html".into()))
        .title("Socketor")
        .transparent(true)
        .build()
        .unwrap();

    #[cfg(target_os = "macos")]
    apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
        .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");
    #[cfg(target_os = "windows")]
    apply_mica(&window, Some(false))
        .expect("Unsupported platform! 'apply_mica' is only supported on Window11");
}