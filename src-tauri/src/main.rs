#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use socketor::services::tcp_server;
use std::thread;
use tauri::{Manager, Window};
#[cfg(target_os = "windows")]
use window_vibrancy::apply_blur;
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};


fn main() {
    let menu = socketor::menu::create_default_menu();

    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "new-instance" => {
                    let app = event.window().app_handle();
                    socketor::menu::create_new_instance(app);
                }
                _ => {}
            }
        })
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");
            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125)))
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tcp_server::start_tcp_server,
            tcp_server::stop_tcp_server,
            init_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn init_process(window: Window) {
    std::thread::spawn(move || {
        loop {
            window.emit("show_test", "Tauri is awesome!").unwrap();
            thread::sleep(std::time::Duration::from_secs(1));
            println!("send message to front");
        }
    });
}