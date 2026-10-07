import SwiftUI
import AppKit

/// Keep native List selection and keyboard handling, but draw selection in the
/// SwiftUI row backgrounds instead of AppKit's accent-colored highlight.
struct NeutralListSelection: NSViewRepresentable {
    func makeNSView(context: Context) -> NeutralListSelectionView {
        NeutralListSelectionView()
    }

    func updateNSView(_ view: NeutralListSelectionView, context: Context) {
        view.updateNow()
        view.scheduleUpdate()
    }

    static func dismantleNSView(_ view: NeutralListSelectionView, coordinator: ()) {
        view.stopObserving()
    }
}

final class NeutralListSelectionView: NSView {
    private var pending = false

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        stopObserving()
        guard window != nil else { return }
        // Selection notifications are synchronous: remove AppKit's highlight
        // before this event is drawn, rather than on the next main-queue turn.
        for name in [NSTableView.selectionIsChangingNotification, NSTableView.selectionDidChangeNotification] {
            NotificationCenter.default.addObserver(self, selector: #selector(selectionChanged(_:)), name: name, object: nil)
        }
        updateNow()
        scheduleUpdate()
    }

    override func layout() {
        super.layout()
        updateNow()
    }

    func stopObserving() {
        NotificationCenter.default.removeObserver(self)
    }

    @objc private func selectionChanged(_ notification: Notification) {
        guard let table = notification.object as? NSTableView,
              let window, table.window === window else { return }
        configure(table)
    }

    func updateNow() {
        guard let root = window?.contentView else { return }
        configure(root)
    }

    func scheduleUpdate() {
        guard !pending else { return }
        pending = true
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.pending = false
            guard let root = self.window?.contentView else { return }
            self.configure(root)
        }
    }

    private func configure(_ view: NSView) {
        if let table = view as? NSTableView {
            if table.selectionHighlightStyle != .none { table.selectionHighlightStyle = .none }
            table.enumerateAvailableRowViews { row, _ in
                if row.isEmphasized { row.isEmphasized = false }
                if row.selectionHighlightStyle != .none { row.selectionHighlightStyle = .none }
            }
        }
        for child in view.subviews { configure(child) }
    }
}

/// Explicit idle hiding, independent of the user's system scroller preference.
struct AutoHidingScrollbars: NSViewRepresentable {
    var alwaysHidden = false

    func makeNSView(context: Context) -> ScrollbarConfigurationView {
        let view = ScrollbarConfigurationView()
        view.alwaysHidden = alwaysHidden
        return view
    }

    func updateNSView(_ view: ScrollbarConfigurationView, context: Context) {
        view.alwaysHidden = alwaysHidden
        view.scheduleConfiguration()
    }

    static func dismantleNSView(_ view: ScrollbarConfigurationView, coordinator: ()) {
        view.stopObserving()
    }
}

/// Keep AppKit's layout/hover updates from revealing an idle overlay scroller.
private final class ActivityScroller: NSScroller {
    override class var isCompatibleWithOverlayScrollers: Bool { true }

    var activityVisible = false {
        didSet {
            super.isHidden = !activityVisible
            super.alphaValue = activityVisible ? 1 : 0
        }
    }

    override var isHidden: Bool {
        get { super.isHidden }
        set { super.isHidden = !activityVisible || newValue }
    }

    override var alphaValue: CGFloat {
        get { super.alphaValue }
        set { super.alphaValue = activityVisible ? newValue : 0 }
    }
}

final class ScrollbarConfigurationView: NSView {
    var alwaysHidden = false
    private final class ScrollState {
        weak var scroll: NSScrollView?
        var origin: NSPoint
        var hideWork: DispatchWorkItem?
        var inputUntil: TimeInterval = 0

        init(_ scroll: NSScrollView) {
            self.scroll = scroll
            origin = scroll.contentView.bounds.origin
        }

        deinit { hideWork?.cancel() }
    }

    private var states: [ObjectIdentifier: ScrollState] = [:]
    private var configurationPending = false
    private var observing = false
    private var inputMonitor: Any?

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        stopObserving()
        guard window != nil else { return }
        observing = true
        inputMonitor = NSEvent.addLocalMonitorForEvents(matching: [.scrollWheel, .keyDown]) { [weak self] event in
            self?.recordScrollInput(event)
            return event
        }
        NotificationCenter.default.addObserver(self, selector: #selector(boundsChanged(_:)),
            name: NSView.boundsDidChangeNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(liveScroll(_:)),
            name: NSScrollView.willStartLiveScrollNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(liveScroll(_:)),
            name: NSScrollView.didLiveScrollNotification, object: nil)
        scheduleConfiguration()
    }

    override func layout() {
        super.layout()
        scheduleConfiguration()
    }

    func stopObserving() {
        if let inputMonitor { NSEvent.removeMonitor(inputMonitor) }
        inputMonitor = nil
        NotificationCenter.default.removeObserver(self)
        observing = false
        states.removeAll()
    }

    func scheduleConfiguration() {
        guard !configurationPending else { return }
        configurationPending = true
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.configurationPending = false
            guard self.observing, let root = self.window?.contentView else { return }
            self.states = self.states.filter { $0.value.scroll?.window === self.window }
            self.configure(root)
        }
    }

    private func configure(_ view: NSView) {
        if let scroll = view as? NSScrollView {
            let state = state(for: scroll)
            scroll.contentView.postsBoundsChangedNotifications = true
            if scroll.scrollerStyle != .overlay { scroll.scrollerStyle = .overlay }
            if alwaysHidden {
                // Helpers never show scrollers, so remove their slots once;
                // unlike idle hiding this policy never changes during scrolling.
                if scroll.hasVerticalScroller { scroll.hasVerticalScroller = false }
                if scroll.hasHorizontalScroller { scroll.hasHorizontalScroller = false }
                var insets = scroll.contentInsets
                if insets.left != 0 || insets.right != 0 {
                    insets.left = 0
                    insets.right = 0
                    scroll.contentInsets = insets
                }
            }
            installActivityScrollers(scroll)
            if state.hideWork == nil { setIndicators(scroll, visible: false) }
        }
        for child in view.subviews { configure(child) }
    }

    private func installActivityScrollers(_ scroll: NSScrollView) {
        func replacement(for original: NSScroller?) -> NSScroller? {
            guard let original, !(original is ActivityScroller) else { return original }
            let scroller = ActivityScroller(frame: original.frame)
            scroller.controlSize = original.controlSize
            scroller.scrollerStyle = .overlay
            scroller.knobStyle = original.knobStyle
            scroller.target = original.target
            scroller.action = original.action
            scroller.doubleValue = original.doubleValue
            scroller.knobProportion = original.knobProportion
            scroller.activityVisible = false
            return scroller
        }
        if let scroller = scroll.verticalScroller, !(scroller is ActivityScroller) {
            scroll.verticalScroller = replacement(for: scroller)
        }
        if let scroller = scroll.horizontalScroller, !(scroller is ActivityScroller) {
            scroll.horizontalScroller = replacement(for: scroller)
        }
    }

    private func state(for scroll: NSScrollView) -> ScrollState {
        let key = ObjectIdentifier(scroll)
        if let state = states[key] { return state }
        let state = ScrollState(scroll)
        states[key] = state
        return state
    }

    private func recordScrollInput(_ event: NSEvent) {
        guard !alwaysHidden, let window, event.window === window else { return }
        let view: NSView?
        if event.type == .scrollWheel, let root = window.contentView {
            view = root.hitTest(root.convert(event.locationInWindow, from: nil))
        } else if event.type == .keyDown, [UInt16(49), 115, 116, 119, 121, 123, 124, 125, 126].contains(event.keyCode) {
            view = window.firstResponder as? NSView
        } else {
            return
        }
        guard let scroll = (view as? NSScrollView) ?? view?.enclosingScrollView else { return }
        state(for: scroll).inputUntil = ProcessInfo.processInfo.systemUptime + 1
    }

    @objc private func boundsChanged(_ notification: Notification) {
        guard let clip = notification.object as? NSClipView,
              let scroll = clip.enclosingScrollView, scroll.window === window else { return }
        let state = state(for: scroll)
        guard state.origin != clip.bounds.origin else { return }
        state.origin = clip.bounds.origin
        installActivityScrollers(scroll)
        // Initial layout and selection restoration also change the clip bounds.
        // Show only for user input or an ongoing scroll, including legacy wheels.
        if state.hideWork != nil || ProcessInfo.processInfo.systemUptime <= state.inputUntil {
            showTemporarily(scroll)
        } else {
            setIndicators(scroll, visible: false)
        }
    }

    @objc private func liveScroll(_ notification: Notification) {
        guard let scroll = notification.object as? NSScrollView,
              scroll.window === window else { return }
        showTemporarily(scroll)
    }

    private func showTemporarily(_ scroll: NSScrollView) {
        guard !alwaysHidden else {
            setIndicators(scroll, visible: false)
            return
        }
        installActivityScrollers(scroll)
        let state = state(for: scroll)
        state.hideWork?.cancel()
        let work = DispatchWorkItem { [weak self, weak state] in
            guard let self, let state, let scroll = state.scroll else { return }
            state.hideWork = nil
            self.setIndicators(scroll, visible: false)
        }
        state.hideWork = work
        setIndicators(scroll, visible: true)
        DispatchQueue.main.asyncAfter(deadline: .now() + 1, execute: work)
    }

    private func setIndicators(_ scroll: NSScrollView, visible: Bool) {
        // Keep scroller installation and content geometry unchanged. Removing
        // scrollers causes SwiftUI lists to remeasure and rewrap their rows.
        let alpha: CGFloat = !alwaysHidden && visible ? 1 : 0
        for scroller in [scroll.verticalScroller, scroll.horizontalScroller] {
            if let scroller {
                (scroller as? ActivityScroller)?.activityVisible = alpha != 0
                // AppKit can restore overlay alpha during hover or layout.
                // Hide the view too, without removing its scroll-view slot.
                scroller.isHidden = alpha == 0
                if scroller.alphaValue != alpha { scroller.alphaValue = alpha }
            }
        }
    }
}

/// Panel boundaries expressed with system semantics rather than paired opaque
/// fills. A separator is one hairline of `separatorColor`, whose alpha adapts to
/// light, dark and high-contrast appearances on its own — no `colorScheme` branch.
struct SystemSeparator: View {
    var vertical = false

    var body: some View {
        Rectangle()
            .fill(Color(nsColor: .separatorColor))
            .frame(width: vertical ? 1 : nil, height: vertical ? nil : 1)
    }
}

/// The content-pane background for the macOS 13 fallback layout, which has no
/// `NavigationSplitView` to supply one.
struct SystemPaneBackground: View {
    var body: some View {
        Color(nsColor: .controlBackgroundColor)
    }
}

/// The system sidebar material, for the macOS 13 fallback layout. `sidebar` is
/// the appearance-tracking material a source list sits on, so the pane reads the
/// same in light, dark and increased-contrast appearances without a colour of its
/// own.
struct SystemSidebarBackground: View {
    var body: some View {
        VisualEffect(material: .sidebar)
    }
}

private struct VisualEffect: NSViewRepresentable {
    let material: NSVisualEffectView.Material

    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = material
        view.blendingMode = .behindWindow
        view.state = .followsWindowActiveState
        return view
    }

    func updateNSView(_ view: NSVisualEffectView, context: Context) {
        view.material = material
    }
}
