import Foundation
import Combine
import Sparkle

// Release tags support only stable, alpha.N and beta.N versions.
struct UpdateVersion: Comparable {
    let core: [UInt64]
    let stage: Int
    let sequence: UInt64

    init?(_ value: String) {
        let parts = value.split(separator: "-", omittingEmptySubsequences: false)
        guard parts.count <= 2 else { return nil }
        let numbers = parts[0].split(separator: ".", omittingEmptySubsequences: false)
        guard numbers.count == 3 else { return nil }
        let core = numbers.compactMap { UInt64($0) }
        guard core.count == 3 else { return nil }
        self.core = core
        if parts.count == 1 {
            stage = 2; sequence = 0
        } else {
            let preview = parts[1].split(separator: ".", omittingEmptySubsequences: false)
            guard preview.count == 2, ["alpha", "beta"].contains(preview[0]),
                  let number = UInt64(preview[1]) else { return nil }
            stage = preview[0] == "alpha" ? 0 : 1
            sequence = number
        }
    }

    static func < (lhs: Self, rhs: Self) -> Bool {
        if lhs.core != rhs.core { return lhs.core.lexicographicallyPrecedes(rhs.core) }
        if lhs.stage != rhs.stage { return lhs.stage < rhs.stage }
        return lhs.sequence < rhs.sequence
    }

    static func permits(_ candidate: String, current: String, channel: String) -> Bool {
        guard let next = Self(candidate), let installed = Self(current), next > installed else { return false }
        switch channel {
        case "stable": return next.stage == 2
        case "beta": return next.stage >= 1
        case "alpha": return true
        default: return false
        }
    }
}

@MainActor
private final class MirrorUpdateDriver: SPUStandardUserDriver {
    var shouldRetry: ((NSError) -> Bool)?

    override func showUpdaterError(_ error: Error, acknowledgement: @escaping () -> Void) {
        if shouldRetry?(error as NSError) == true {
            acknowledgement()
        } else {
            super.showUpdaterError(error, acknowledgement: acknowledgement)
        }
    }
}

/// Sparkle verifies the update archive, replaces the bundle and relaunches the app.
@MainActor
final class Updater: NSObject, ObservableObject, SPUUpdaterDelegate {
    @Published private var ready = false
    var canCheck: Bool { ready && pendingFallback == nil && !UpdateActivity.isBusy }
    @Published var automaticChecks: Bool {
        didSet {
            UserDefaults.standard.set(automaticChecks, forKey: "SUEnableAutomaticChecks")
            if updater?.automaticallyChecksForUpdates != automaticChecks {
                updater?.automaticallyChecksForUpdates = automaticChecks
            }
        }
    }
    @Published var channel: String {
        didSet { UserDefaults.standard.set(channel, forKey: "updateChannel"); updater?.resetUpdateCycle() }
    }
    @Published var automaticDownloads: Bool {
        didSet {
            UserDefaults.standard.set(automaticDownloads, forKey: "SUAutomaticallyUpdate")
            if updater?.automaticallyDownloadsUpdates != automaticDownloads {
                updater?.automaticallyDownloadsUpdates = automaticDownloads
            }
        }
    }
    @Published private(set) var unavailableReason: String?
    private var updater: SPUUpdater?
    private var observations = Set<AnyCancellable>()
    private let baseURL: String
    private let fallbackURL = "https://github.com/tangxiangong/bibcitex/releases/download/update-feed"
    private var usingFallback = false
    private var pendingFallback: SPUUpdateCheck?
    private var checkedChannel = ""
    private var checkedLanguage = ""
    private var pendingRelaunch: (() -> Void)?
    private var resumeTask: Task<Void, Never>?

    override init() {
        let version = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""
        let initial = version.contains("-alpha.") ? "alpha" : version.contains("-beta.") ? "beta" : "stable"
        let saved = UserDefaults.standard.string(forKey: "updateChannel") ?? initial
        channel = ["stable", "beta", "alpha"].contains(saved) ? saved : initial
        automaticChecks = UserDefaults.standard.object(forKey: "SUEnableAutomaticChecks") as? Bool ?? true
        automaticDownloads = UserDefaults.standard.bool(forKey: "SUAutomaticallyUpdate")
        baseURL = Bundle.main.object(forInfoDictionaryKey: "UpdateBaseURL") as? String
            ?? "https://app-release-1302963684.cos.ap-guangzhou.myqcloud.com/bibcitex/update-feed"
        super.init()
        UserDefaults.standard.set(channel, forKey: "updateChannel")
        guard let url = URL(string: baseURL), url.scheme == "https", url.host?.isEmpty == false else { unavailableReason = "更新地址无效"; return }
        guard let key = Bundle.main.object(forInfoDictionaryKey: "SUPublicEDKey") as? String,
              Data(base64Encoded: key)?.count == 32 else { unavailableReason = "此构建未配置更新验证公钥"; return }
        let driver = MirrorUpdateDriver(hostBundle: .main, delegate: nil)
        driver.shouldRetry = { [weak self] error in
            guard let self else { return false }
            return !self.usingFallback && self.checkedChannel == self.channel
                && self.checkedLanguage == L10n.language && Self.canRetryFromMirror(error)
        }
        let updater = SPUUpdater(hostBundle: .main, applicationBundle: .main, userDriver: driver, delegate: self)
        self.updater = updater
        updater.automaticallyChecksForUpdates = automaticChecks
        updater.updateCheckInterval = 6 * 60 * 60
        updater.automaticallyDownloadsUpdates = automaticDownloads
        do { try updater.start() }
        catch { unavailableReason = error.localizedDescription; return }
        updater.publisher(for: \.canCheckForUpdates)
            .receive(on: RunLoop.main).sink { [weak self] ready in
                self?.ready = ready
                self?.retryFallbackIfReady()
            }
            .store(in: &observations)
        updater.publisher(for: \.automaticallyChecksForUpdates)
            .receive(on: RunLoop.main).sink { [weak self] enabled in
                if self?.automaticChecks != enabled { self?.automaticChecks = enabled }
            }.store(in: &observations)
        UpdateActivity.changed.receive(on: RunLoop.main).sink { [weak self] in
            self?.objectWillChange.send()
            self?.retryFallbackIfReady()
        }.store(in: &observations)
        updater.publisher(for: \.automaticallyDownloadsUpdates)
            .receive(on: RunLoop.main).sink { [weak self] enabled in
                if self?.automaticDownloads != enabled { self?.automaticDownloads = enabled }
            }.store(in: &observations)
    }

    func feedURLString(for updater: SPUUpdater) -> String? {
        #if arch(arm64)
        let architecture = "arm64"
        #else
        let architecture = "x86_64"
        #endif
        checkedChannel = channel
        checkedLanguage = L10n.language
        let source = usingFallback ? fallbackURL : baseURL
        return "\(source)/appcast-\(architecture)-\(channel)-\(L10n.language).xml"
    }
    func bestValidUpdate(in appcast: SUAppcast, for updater: SPUUpdater) -> SUAppcastItem? {
        let current = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""
        // Build numbers remain Sparkle's installation ordering; also reject semantic downgrades
        // when a later maintenance build is published after a newer prerelease.
        return appcast.items.filter {
            UpdateVersion.permits($0.displayVersionString, current: current, channel: channel)
        }.max {
            SUStandardVersionComparator.default.compareVersion($0.versionString, toVersion: $1.versionString) == .orderedAscending
        } ?? SUAppcastItem.empty()
    }
    func updater(_ updater: SPUUpdater, didFinishUpdateCycleFor updateCheck: SPUUpdateCheck, error: Error?) {
        defer { if pendingFallback == nil { usingFallback = false } }
        guard !usingFallback, checkedChannel == channel, checkedLanguage == L10n.language,
              let failure = error as NSError?, Self.canRetryFromMirror(failure) else { return }
        usingFallback = true
        pendingFallback = updateCheck
        // Sparkle must finish tearing down its driver before another check starts.
        DispatchQueue.main.async { [weak self] in self?.retryFallbackIfReady() }
    }

    private static func canRetryFromMirror(_ error: NSError) -> Bool {
        if error.domain == NSURLErrorDomain { return error.code != NSURLErrorCancelled }
        if let underlying = error.userInfo[NSUnderlyingErrorKey] as? NSError,
           underlying.domain == NSURLErrorDomain, underlying.code == NSURLErrorCancelled { return false }
        // Only feed/transport failures: never retry cancellation, installation or signature errors.
        return error.domain == SUSparkleErrorDomain && [1000, 1002, 2001].contains(error.code)
    }

    private func retryFallbackIfReady() {
        guard let check = pendingFallback, let updater = updater,
              updater.canCheckForUpdates, !UpdateActivity.isBusy else { return }
        pendingFallback = nil
        guard checkedChannel == channel, checkedLanguage == L10n.language else {
            usingFallback = false
            return
        }
        guard check != .updatesInBackground || automaticChecks else { usingFallback = false; return }
        ready = false
        switch check {
        case .updates: updater.checkForUpdates()
        case .updatesInBackground: updater.checkForUpdatesInBackground()
        case .updateInformation: updater.checkForUpdateInformation()
        @unknown default: usingFallback = false
        }
    }

    func languageChanged() { updater?.resetUpdateCycle() }
    func check() { guard canCheck, pendingFallback == nil else { return }; updater?.checkForUpdates() }

    func updater(_ updater: SPUUpdater, mayPerform updateCheck: SPUUpdateCheck) throws {
        if updateCheck == .updatesInBackground && !automaticChecks {
            throw NSError(domain: NSURLErrorDomain, code: NSURLErrorCancelled)
        }
        if UpdateActivity.isBusy {
            throw NSError(domain: "com.bibcitex.updates", code: 1,
                          userInfo: [NSLocalizedDescriptionKey: L10n.text("请等待当前操作完成后检查更新。")])
        }
    }

    func updater(_ updater: SPUUpdater, shouldPostponeRelaunchForUpdate item: SUAppcastItem,
                 untilInvokingBlock installHandler: @escaping () -> Void) -> Bool {
        guard UpdateActivity.isBusy else { return false }
        pendingRelaunch = installHandler
        resumeTask?.cancel()
        resumeTask = Task { [weak self] in
            while self?.pendingRelaunch != nil {
                do { try await Task.sleep(for: .seconds(1)) }
                catch { return }
                guard !UpdateActivity.isBusy, let install = self?.pendingRelaunch else { continue }
                self?.pendingRelaunch = nil
                self?.resumeTask = nil
                install()
            }
        }
        return true
    }
}

/// Includes service calls made by all helper and workbench actors.
enum UpdateActivity {
    private static let lock = NSLock()
    private static var active = 0
    static let changed = PassthroughSubject<Void, Never>()
    static var isBusy: Bool { lock.withLock { active != 0 } }

    static func perform<T>(_ operation: () throws -> T) rethrows -> T {
        lock.withLock { active += 1 }
        changed.send()
        defer { lock.withLock { active -= 1 }; changed.send() }
        return try operation()
    }
}
