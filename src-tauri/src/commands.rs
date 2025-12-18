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
    xpaste::focus_previous_window,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

/// Load references from a bib file
fn load_references(path: impl AsRef<std::path::Path>) -> Result<Vec<Reference>> {
    let bib = parse(path)?;
    Ok(bib.iter().map(Reference::from).collect())
}

// Global settings state
static SETTINGS: Mutex<Option<Setting>> = Mutex::new(None);

// Helper window 选择的文献库 (持久化在内存中)
static HELPER_BIB: Mutex<Option<(String, String)>> = Mutex::new(None); // (name, path)

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
    HELPER_BIB.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_helper_bib(name: String, path: String) {
    *HELPER_BIB.lock().unwrap() = Some((name, path));
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
    match field.as_str() {
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
    let mut clipboard = arboard::Clipboard::new().map_err(|e| Error::Clipboard(e.to_string()))?;
    clipboard
        .set_text(&text)
        .map_err(|e| Error::Clipboard(e.to_string()))
}

#[tauri::command]
pub fn paste_to_app(text: String) -> Result<()> {
    // First copy to clipboard
    copy_to_clipboard(text)?;
    // Then focus previous window and paste
    focus_previous_window().map_err(|e| Error::Clipboard(e.to_string()))
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

// Update commands
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<serde_json::Value> {
    use tauri_plugin_updater::UpdaterExt;

    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => Ok(serde_json::json!({
                "available": true,
                "version": update.version,
                "notes": update.body
            })),
            Ok(None) => Ok(serde_json::json!({
                "available": false
            })),
            Err(e) => Ok(serde_json::json!({
                "available": false,
                "error": e.to_string()
            })),
        },
        Err(e) => Ok(serde_json::json!({
            "available": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<()> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|e| Error::Tauri(e.to_string()))?;

    if let Some(update) = updater
        .check()
        .await
        .map_err(|e| Error::Tauri(e.to_string()))?
    {
        update
            .download_and_install(|_, _| {}, || {})
            .await
            .map_err(|e| Error::Tauri(e.to_string()))?;
    }

    Ok(())
}

// Helper window
pub fn create_helper_window(app: AppHandle) -> Result<()> {
    // Check if helper window already exists
    if let Some(window) = app.get_webview_window("helper") {
        // If it exists, close it (toggle behavior)
        let _ = window.close();
        return Ok(());
    }

    let width = 900.0; // 增加宽度以容纳更多内容
    let height = 60.0;

    // 获取主显示器信息以计算位置
    let monitor = app.primary_monitor().ok().flatten();

    let (screen_width, screen_height) = monitor
        .as_ref()
        .map(|m| {
            let size = m.size();
            let scale = m.scale_factor();
            (size.width as f64 / scale, size.height as f64 / scale)
        })
        .unwrap_or((1920.0, 1080.0));

    // 窗口居中水平，垂直位置在屏幕中间靠上（1/3 处）
    let x = (screen_width - width) / 2.0;
    let y = screen_height / 3.0 - height / 2.0;

    use tauri::WebviewWindowBuilder;

    let window_builder =
        WebviewWindowBuilder::new(&app, "helper", WebviewUrl::App("/helper".into()))
            .title("BibCiTeX Helper")
            .inner_size(width, height)
            .min_inner_size(width, 60.0)
            .max_inner_size(width, 600.0)
            .position(x, y)
            .decorations(false)
            .always_on_top(true)
            .resizable(true)
            .skip_taskbar(true)
            .shadow(true);

    let window: WebviewWindow = window_builder
        .build()
        .map_err(|e: tauri::Error| Error::Tauri(e.to_string()))?;

    // 跨平台圆角效果通过 CSS 实现
    // 无装饰窗口 + CSS 圆角 = 视觉上的圆角窗口

    let _ = window.set_focus();

    // Close window when it loses focus
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = window_clone.close();
        }
    });

    Ok(())
}

// Resize helper window
#[tauri::command]
pub fn resize_helper_window(app: AppHandle, height: f64) -> Result<()> {
    if let Some(window) = app.get_webview_window("helper") {
        let new_height = height.clamp(60.0, 600.0);
        // 直接调整大小，不切换 resizable 状态以避免焦点丢失
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 900.0, // 保持与创建时相同的宽度
            height: new_height,
        }));
    }
    Ok(())
}

// Open helper window (called from frontend)
#[tauri::command]
pub fn open_helper_window(app: AppHandle) -> Result<()> {
    create_helper_window(app)
}
