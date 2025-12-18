mod commands;
mod error;

pub use error::{Error, Result};

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        // Handle global shortcut (Cmd+Shift+K or Super+Shift+K)
                        if shortcut.key == tauri_plugin_global_shortcut::Code::KeyK {
                            if let Some(window) = app.get_webview_window("helper") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            } else {
                                // Create helper window
                                let _ = commands::create_helper_window(app.clone());
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Start the app observer for cross-app paste
            xpaste::observe_app();

            // Register global shortcut
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;

                let shortcut = if cfg!(target_os = "macos") {
                    "Command+Shift+K"
                } else {
                    "Super+Shift+K"
                };

                app.global_shortcut().register(shortcut)?;
            }

            // Setup system tray
            #[cfg(desktop)]
            {
                use tauri::{
                    menu::{Menu, MenuItem},
                    tray::TrayIconBuilder,
                };

                let quit = MenuItem::with_id(app, "quit", "退出 BibCiTeX", true, None::<&str>)?;
                let helper = MenuItem::with_id(app, "helper", "快捷助手", true, None::<&str>)?;
                let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;

                let menu = Menu::with_items(app, &[&show, &helper, &quit])?;

                TrayIconBuilder::new()
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "helper" => {
                            let _ = commands::create_helper_window(app.clone());
                        }
                        _ => {}
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_settings,
            commands::save_settings,
            commands::add_bibliography,
            commands::remove_bibliography,
            commands::load_bibliography,
            commands::parse_bib_file,
            commands::search_references,
            commands::search_by_field,
            commands::copy_to_clipboard,
            commands::paste_to_app,
            commands::select_bib_file,
            commands::open_url,
            commands::open_file,
            commands::check_update,
            commands::install_update,
            commands::resize_helper_window,
            commands::open_helper_window,
            commands::get_helper_bib,
            commands::set_helper_bib,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
