import Foundation
import Combine
import Sparkle

/// Sparkle owns native presentation, signature validation, installation and relaunch.
@MainActor
final class Updater: ObservableObject {
    @Published private(set) var canCheck = false
    private var controller: SPUStandardUpdaterController?
    private var observation: AnyCancellable?
    init() {
        guard let feed = Bundle.main.object(forInfoDictionaryKey: "SUFeedURL") as? String,
              let url = URL(string: feed), url.scheme == "https",
              let publicKey = Bundle.main.object(forInfoDictionaryKey: "SUPublicEDKey") as? String,
              !publicKey.isEmpty else { return }
        let controller = SPUStandardUpdaterController(startingUpdater: true, updaterDelegate: nil, userDriverDelegate: nil)
        self.controller = controller
        observation = controller.updater.publisher(for: \.canCheckForUpdates)
            .receive(on: RunLoop.main).sink { [weak self] in self?.canCheck = $0 }
    }
    func check() { guard canCheck else { return }; controller?.checkForUpdates(nil) }
}
