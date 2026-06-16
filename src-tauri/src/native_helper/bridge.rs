use std::os::raw::{c_char, c_int};

use crate::native_helper::ffi::{
    FfiBibliography, FfiBibliographyArray, FfiReferenceArray, bool_to_c_int, cstring_to_str,
};
use crate::native_helper::{ffi, state};

#[cfg(target_os = "macos")]
#[link(name = "NativeHelper", kind = "static")]
unsafe extern "C" {}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn bibcitex_native_helper_show();
    fn bibcitex_native_helper_hide();
    fn bibcitex_native_helper_set_theme(theme: c_int);
    fn bibcitex_native_helper_is_visible() -> c_int;
}

pub fn show_helper() {
    #[cfg(target_os = "macos")]
    unsafe {
        bibcitex_native_helper_show();
    }
}

pub fn hide_helper() {
    #[cfg(target_os = "macos")]
    unsafe {
        bibcitex_native_helper_hide();
    }
}

pub fn set_theme(theme: crate::native_helper::ThemeMode) {
    #[cfg(target_os = "macos")]
    unsafe {
        bibcitex_native_helper_set_theme(theme as c_int);
    }
}

pub fn is_panel_visible() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        bibcitex_native_helper_is_visible() != 0
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn bibcitex_native_helper_init_list() -> FfiBibliographyArray {
    state::list_bibliographies_ffi()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_native_helper_set_current_bibliography(
    name: *const c_char,
    path: *const c_char,
) -> c_int {
    let name = cstring_to_str(name).unwrap_or_default();
    let path = cstring_to_str(path).unwrap_or_default();
    match state::set_current_bibliography(name, path) {
        Ok(_) => {
            state::clear_last_error();
            bool_to_c_int(true)
        }
        Err(error) => {
            state::set_last_error(error);
            bool_to_c_int(false)
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_native_helper_current_bibliography() -> FfiBibliography {
    match state::current_bibliography() {
        Some((name, path)) => {
            let mut updated_at = String::new();
            let mut description = None;
            for (current_name, current_path, current_updated_at, current_description) in
                state::list_bibliographies()
            {
                if current_name == name && current_path == path {
                    updated_at = current_updated_at;
                    description = current_description;
                    break;
                }
            }
            if updated_at.is_empty() {
                updated_at = crate::native_helper::current_timestamp();
            }
            ffi::to_ffi_bibliography(name, path, updated_at, description)
        }
        None => FfiBibliography {
            name: ffi::FfiString::null(),
            path: ffi::FfiString::null(),
            updated_at: ffi::FfiString::null(),
            description: ffi::FfiString::null(),
            has_description: false,
        },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_native_helper_search_references(
    query: *const c_char,
) -> FfiReferenceArray {
    let query = cstring_to_str(query).unwrap_or_default();
    let references = state::search_references(&query);
    ffi::references_to_ffi_array(references)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_native_helper_last_error() -> ffi::FfiString {
    match state::get_last_error() {
        Some(error) => ffi::to_ffi_string(Some(error)),
        None => ffi::to_ffi_string(None),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_native_helper_copy_and_paste(cite_key: *const c_char) -> c_int {
    let cite_key = cstring_to_str(cite_key).unwrap_or_default();

    if cite_key.trim().is_empty() {
        state::set_last_error("cite key 为空".to_string());
        return bool_to_c_int(false);
    }

    match crate::commands::paste_reference_key(&cite_key) {
        Ok(_) => {
            state::clear_last_error();
            bool_to_c_int(true)
        }
        Err(error) => {
            let text = error.to_string();
            state::set_last_error(text.clone());
            bool_to_c_int(false)
        }
    }
}
