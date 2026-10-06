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

/// Sparkle verifies the update archive, replaces the bundle and relaunches the app.
@MainActor
final class Updater: NSObject, ObservableObject, SPUUpdaterDelegate {
    @Published private(set) var canCheck = false
    @Published var channel: String {
        didSet { UserDefaults.standard.set(channel, forKey: "updateChannel"); controller?.updater.resetUpdateCycle() }
    }
    @Published var automaticDownloads: Bool {
        didSet {
            UserDefaults.standard.set(automaticDownloads, forKey: "SUAutomaticallyUpdate")
            if controller?.updater.automaticallyDownloadsUpdates != automaticDownloads {
                controller?.updater.automaticallyDownloadsUpdates = automaticDownloads
            }
        }
    }
    private var controller: SPUStandardUpdaterController?
    private var observations = Set<AnyCancellable>()
    private let baseURL: String

    override init() {
        let version = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""
        let initial = version.contains("-alpha.") ? "alpha" : version.contains("-beta.") ? "beta" : "stable"
        let saved = UserDefaults.standard.string(forKey: "updateChannel") ?? initial
        channel = ["stable", "beta", "alpha"].contains(saved) ? saved : initial
        automaticDownloads = UserDefaults.standard.bool(forKey: "SUAutomaticallyUpdate")
        baseURL = Bundle.main.object(forInfoDictionaryKey: "UpdateBaseURL") as? String
            ?? "https://github.com/tangxiangong/bibcitex/releases/download/update-feed"
        super.init()
        UserDefaults.standard.set(channel, forKey: "updateChannel")
        guard URL(string: baseURL)?.scheme == "https",
              let key = Bundle.main.object(forInfoDictionaryKey: "SUPublicEDKey") as? String,
              !key.isEmpty else { return }
        let controller = SPUStandardUpdaterController(startingUpdater: false, updaterDelegate: self, userDriverDelegate: nil)
        self.controller = controller
        controller.updater.automaticallyChecksForUpdates = true
        controller.updater.updateCheckInterval = 6 * 60 * 60
        controller.updater.automaticallyDownloadsUpdates = automaticDownloads
        controller.startUpdater()
        controller.updater.publisher(for: \.canCheckForUpdates)
            .receive(on: RunLoop.main).sink { [weak self] in self?.canCheck = $0 }
            .store(in: &observations)
        controller.updater.publisher(for: \.automaticallyDownloadsUpdates)
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
        return "\(baseURL)/appcast-\(architecture)-\(channel)-\(L10n.language).xml"
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
    func languageChanged() { controller?.updater.resetUpdateCycle() }
    func check() { guard canCheck else { return }; controller?.checkForUpdates(nil) }
}
