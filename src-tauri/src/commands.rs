use crate::{
    Error, Result,
    core::{
        bib::{Reference, parse},
        search::{
            search_references as search_references_impl, search_references_by_author,
            search_references_by_journal, search_references_by_title, search_references_by_year,
        },
        setting::{BibliographyInfo, Setting},
    },
    native_helper::{ThemeMode, bridge as native_helper_bridge, state as native_helper_state},
    xpaste::focus_previous_window,
};
use arboard::Clipboard;
use std::path::PathBuf;
use std::sync::Mutex;
#[cfg(target_os = "macos")]
use tauri::AppHandle;
#[cfg(not(target_os = "macos"))]
use tauri::{AppHandle, Emitter, Manager, WebviewUrl};

use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

fn set_clipboard_text(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().map_err(|e| Error::Clipboard(e.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|e| Error::Clipboard(e.to_string()))
}

pub(crate) fn paste_reference_key(cite_key: &str) -> Result<()> {
    set_clipboard_text(cite_key)?;
    focus_previous_window().map_err(|e| Error::Clipboard(e.to_string()))
}

/// Load references from a bib file
fn load_references(path: impl AsRef<std::path::Path>) -> Result<Vec<Reference>> {
    let bib = parse(path)?;
    Ok(bib.iter().map(Reference::from).collect())
}

// Global settings state
static SETTINGS: Mutex<Option<Setting>> = Mutex::new(None);

#[cfg(not(target_os = "macos"))]
const HELPER_WIDTH: f64 = 760.0;
#[cfg(not(target_os = "macos"))]
const HELPER_MIN_HEIGHT: f64 = 70.0;
#[cfg(not(target_os = "macos"))]
const HELPER_DEFAULT_HEIGHT: f64 = 400.0;
#[cfg(not(target_os = "macos"))]
const HELPER_MAX_HEIGHT: f64 = 2000.0;

#[cfg(not(target_os = "macos"))]
fn helper_webview_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window("helper")
}

fn notify_helper_opened(_app: &AppHandle) {
    #[cfg(not(target_os = "macos"))]
    if let Some(window) = helper_webview_window(_app) {
        let _ = window.emit("helper-opened", ());
    }
}

fn get_settings() -> Setting {
    let mut settings = SETTINGS.lock().unwrap();
    if settings.is_none() {
        *settings = Some(Setting::load());
    }
    settings.clone().unwrap()
}

fn set_settings(new_settings: Setting) {
    let mut settings = SETTINGS.lock().unwrap();
    *settings = Some(new_settings);
}

// Helper bib persistence
#[tauri::command]
pub fn get_helper_bib() -> Option<(String, String)> {
    native_helper_state::current_bibliography()
}

#[tauri::command]
pub fn set_helper_bib(name: String, path: String) -> Result<()> {
    native_helper_state::set_current_bibliography(name, path)
        .map(|_| ())
        .map_err(Error::NativeHelper)?;
    Ok(())
}

// Settings commands
#[tauri::command]
pub fn load_settings() -> Result<Setting> {
    Ok(get_settings())
}

#[tauri::command]
pub fn save_settings(settings: Setting) -> Result<()> {
    let s = settings.clone();
    s.update_file()?;
    set_settings(s);
    Ok(())
}

#[tauri::command]
pub fn add_bibliography(
    name: String,
    path: String,
    description: Option<String>,
) -> Result<Option<BibliographyInfo>> {
    let mut settings = get_settings();
    let result = settings.add_update_bibliography(&name, PathBuf::from(path), description)?;
    settings.update_file()?;
    set_settings(settings);
    Ok(result)
}

#[tauri::command]
pub fn remove_bibliography(name: String) -> Result<Option<BibliographyInfo>> {
    let mut settings = get_settings();
    let result = settings.remove_bibliography(&name);
    settings.update_file()?;
    set_settings(settings);
    Ok(result)
}

// Bibliography commands
#[tauri::command]
pub fn load_bibliography(path: String) -> Result<Vec<Reference>> {
    load_references(path)
}

#[tauri::command]
pub fn parse_bib_file(path: String) -> Result<Vec<Reference>> {
    load_references(path)
}

// Search commands
#[tauri::command]
pub fn search_references(references: Vec<Reference>, query: String) -> Vec<Reference> {
    search_references_impl(&references, &query)
}

#[tauri::command]
pub fn search_by_field(references: Vec<Reference>, query: String, field: String) -> Vec<Reference> {
    match field.to_lowercase().as_str() {
        "author" => search_references_by_author(&references, &query),
        "title" => search_references_by_title(&references, &query),
        "journal" => search_references_by_journal(&references, &query),
        "year" => search_references_by_year(&references, &query),
        _ => search_references_impl(&references, &query),
    }
}

// Clipboard commands
#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<()> {
    set_clipboard_text(&text)
}

#[tauri::command]
pub fn paste_to_app(text: String) -> Result<()> {
    paste_reference_key(&text)
}

// File dialog command
#[tauri::command]
pub async fn select_bib_file(app: AppHandle) -> Result<Option<String>> {
    let file = app
        .dialog()
        .file()
        .add_filter("BibTeX", &["bib", "txt"])
        .set_title("选择文献库文件")
        .blocking_pick_file();

    Ok(file.map(|f| f.to_string()))
}

// Utility commands
#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> Result<()> {
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| Error::Tauri(e.to_string()))
}

#[tauri::command]
pub async fn open_file(app: AppHandle, path: String) -> Result<()> {
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| Error::Tauri(e.to_string()))
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn set_native_helper_theme(theme: String) {
    let native_theme = match theme.as_str() {
        "mocha" => ThemeMode::Dark,
        _ => ThemeMode::Light,
    };
    native_helper_bridge::set_theme(native_theme);
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub fn set_native_helper_theme(_theme: String) {}

// Helper window — macOS panel management via global static
#[cfg(target_os = "macos")]
pub fn toggle_helper_panel(app: &AppHandle) {
    if native_helper_bridge::is_panel_visible() {
        native_helper_bridge::hide_helper();
    } else {
        native_helper_bridge::show_helper();
        notify_helper_opened(app);
    }
}

#[cfg(target_os = "macos")]
pub fn hide_helper_panel() {
    native_helper_bridge::hide_helper();
}

#[cfg(target_os = "macos")]
pub fn create_helper_window(app: AppHandle) -> Result<()> {
    let _ = app;
    native_helper_bridge::show_helper();
    notify_helper_opened(&app);

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn create_helper_window(app: AppHandle) -> Result<()> {
    // If already exists, toggle visibility
    if let Some(window) = app.get_webview_window("helper") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            notify_helper_opened(&app);
        }
        return Ok(());
    }

    let width = HELPER_WIDTH;
    let height = HELPER_DEFAULT_HEIGHT;

    let monitor = app.primary_monitor().ok().flatten();
    let (screen_width, screen_height) = monitor
        .as_ref()
        .map(|m| {
            let size = m.size();
            let scale = m.scale_factor();
            (size.width as f64 / scale, size.height as f64 / scale)
        })
        .unwrap_or((1920.0, 1080.0));

    let x = (screen_width - width) / 2.0;
    let y = screen_height / 3.0 - height / 2.0;

    use tauri::WebviewWindowBuilder;

    let window = WebviewWindowBuilder::new(&app, "helper", WebviewUrl::App("/helper".into()))
        .title("BibCiTeX 助手")
        .inner_size(width, height)
        .min_inner_size(HELPER_WIDTH, HELPER_MIN_HEIGHT)
        .max_inner_size(HELPER_WIDTH, HELPER_MAX_HEIGHT)
        .position(x, y)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .resizable(true)
        .skip_taskbar(true)
        .shadow(true)
        .build()
        .map_err(|e: tauri::Error| Error::Tauri(e.to_string()))?;

    let _ = window.set_focus();
    notify_helper_opened(&app);

    // Hide (not close) when it loses focus
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = window_clone.hide();
        }
    });

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn apply_helper_window_height(window: &tauri::WebviewWindow, height: f64) {
    let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
        width: HELPER_WIDTH,
        height: HELPER_MIN_HEIGHT,
    })));
    let _ = window.set_max_size(Some(tauri::Size::Logical(tauri::LogicalSize {
        width: HELPER_WIDTH,
        height: HELPER_MAX_HEIGHT,
    })));
    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
        width: HELPER_WIDTH,
        height,
    }));
}

// Resize helper window
#[tauri::command]
pub fn resize_helper_window(app: AppHandle, height: f64) -> Result<()> {
    let _ = app;
    let _ = height;
    #[cfg(not(target_os = "macos"))]
    {
        let new_height = height.clamp(HELPER_MIN_HEIGHT, HELPER_MAX_HEIGHT);
        if let Some(window) = helper_webview_window(&app) {
            apply_helper_window_height(&window, new_height);
        }
    }

    Ok(())
}

// Open helper window (called from frontend)
#[tauri::command]
pub fn open_helper_window(app: AppHandle) -> Result<()> {
    create_helper_window(app)
}

// Hide helper window (called from frontend for Esc key)
#[tauri::command]
pub fn hide_helper_window(_app: AppHandle) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        hide_helper_panel();
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = _app.get_webview_window("helper") {
            let _ = window.hide();
        }
    }

    Ok(())
}
