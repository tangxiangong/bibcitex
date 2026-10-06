import Foundation
import Combine
import Sparkle

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
    func languageChanged() { controller?.updater.resetUpdateCycle() }
    func check() { guard canCheck else { return }; controller?.checkForUpdates(nil) }
}
