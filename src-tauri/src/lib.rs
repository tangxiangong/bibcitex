mod commands;
mod error;

pub mod core;
pub mod xpaste;
pub use error::{Error, Result};

use tauri::Manager;

#[cfg(target_os = "macos")]
use tauri_nspanel::ManagerExt as NSPanelManagerExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
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
                            #[cfg(target_os = "macos")]
                            {
                                // On macOS, use nspanel
                                if let Ok(panel) = app.get_webview_panel("helper") {
                                    if panel.is_visible() {
                                        panel.hide();
                                    } else {
                                        panel.show();
                                    }
                                } else {
                                    let _ = commands::create_helper_window(app.clone());
                                }
                            }
                            #[cfg(not(target_os = "macos"))]
                            {
                                if let Some(window) = app.get_webview_window("helper") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                } else {
                                    let _ = commands::create_helper_window(app.clone());
                                }
                            }
                        }
                    }
                })
                .build(),
        );
    // Initialize nspanel plugin on macOS
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
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
                    Emitter,
                    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
                    tray::TrayIconBuilder,
                };

                let quit = MenuItem::with_id(app, "quit", "退出 BibCiTeX", true, None::<&str>)?;
                let helper = MenuItem::with_id(app, "helper", "快捷助手", true, None::<&str>)?;
                let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;

                let tray_menu = Menu::with_items(app, &[&show, &helper, &quit])?;

                TrayIconBuilder::new()
                    .menu(&tray_menu)
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
                            #[cfg(target_os = "macos")]
                            {
                                if let Ok(panel) = app.get_webview_panel("helper") {
                                    if panel.is_visible() {
                                        panel.hide();
                                    } else {
                                        panel.show();
                                    }
                                } else {
                                    let _ = commands::create_helper_window(app.clone());
                                }
                            }
                            #[cfg(not(target_os = "macos"))]
                            {
                                let _ = commands::create_helper_window(app.clone());
                            }
                        }
                        _ => {}
                    })
                    .build(app)?;

                // Application Menu
                let app_menu = Submenu::with_items(
                    app,
                    "App",
                    true,
                    &[
                        &PredefinedMenuItem::about(app, None, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(
                            app,
                            "check_update",
                            "Check for Updates...",
                            true,
                            None::<&str>,
                        )?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::services(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::hide(app, None)?,
                        &PredefinedMenuItem::hide_others(app, None)?,
                        &PredefinedMenuItem::show_all(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::quit(app, None)?,
                    ],
                )?;

                let edit_menu = Submenu::with_items(
                    app,
                    "Edit",
                    true,
                    &[
                        &PredefinedMenuItem::undo(app, None)?,
                        &PredefinedMenuItem::redo(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::cut(app, None)?,
                        &PredefinedMenuItem::copy(app, None)?,
                        &PredefinedMenuItem::paste(app, None)?,
                        &PredefinedMenuItem::select_all(app, None)?,
                    ],
                )?;

                let menu = Menu::with_items(app, &[&app_menu, &edit_menu])?;
                app.set_menu(menu)?;

                app.on_menu_event(|app, event| {
                    if event.id.as_ref() == "check_update"
                        && app.emit_to("main", "check-update-trigger", ()).is_err()
                    {
                        let _ = app.emit("check-update-trigger", ());
                    }
                });
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
