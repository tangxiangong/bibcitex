import Foundation

@_cdecl("bibcitex_native_helper_show")
public func bibcitex_native_helper_show() {
    DispatchQueue.main.async {
        NativeHelperPanelController.shared.showPanel()
    }
}

@_cdecl("bibcitex_native_helper_is_visible")
public func bibcitex_native_helper_is_visible() -> Int32 {
    let readVisibility = {
        MainActor.assumeIsolated {
            NativeHelperPanelController.shared.isPanelVisible()
        }
    }

    if Thread.isMainThread {
        return readVisibility() ? 1 : 0
    }

    var isVisible = false
    DispatchQueue.main.sync {
        isVisible = readVisibility()
    }
    return isVisible ? 1 : 0
}

@_cdecl("bibcitex_native_helper_hide")
public func bibcitex_native_helper_hide() {
    DispatchQueue.main.async {
        NativeHelperPanelController.shared.hidePanel()
    }
}

@_cdecl("bibcitex_native_helper_set_theme")
public func bibcitex_native_helper_set_theme(_ theme: Int32) {
    DispatchQueue.main.async {
        NativeHelperPanelController.shared.setTheme(theme)
    }
}
