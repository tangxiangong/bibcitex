import Foundation

@main
struct UpdatePolicyRegression {
    static func main() {
        let ordered = ["1.0.0-alpha.2", "1.0.0-alpha.10", "1.0.0-beta.1", "1.0.0-beta.10", "1.0.0", "1.0.1", "1.1.0-alpha.1", "2.0.0"]
        for (index, value) in ordered.enumerated() {
            for (otherIndex, other) in ordered.enumerated() {
                precondition(UpdateVersion.permits(value, current: other, channel: "alpha") == (index > otherIndex))
            }
        }
        precondition(!UpdateVersion.permits("1.0.1", current: "1.1.0-beta.1", channel: "stable"))
        precondition(UpdateVersion.permits("1.1.0", current: "1.1.0-beta.1", channel: "stable"))
        precondition(!UpdateVersion.permits("2.0.0-alpha.1", current: "1.0.0", channel: "beta"))
        precondition(UpdateVersion.permits("2.0.0-beta.1", current: "1.0.0", channel: "beta"))
        precondition(!UpdateVersion.permits("2.0.0-beta.1", current: "1.0.0", channel: "stable"))
        for invalid in ["", "1.0", "1.0.0-rc.1", "1.0.0-beta", "1.0.0-beta.x", "1.0.0-beta.1-extra"] {
            precondition(!UpdateVersion.permits(invalid, current: "1.0.0", channel: "alpha"))
            precondition(!UpdateVersion.permits("2.0.0", current: invalid, channel: "alpha"))
        }
        precondition(!UpdateVersion.permits("2.0.0", current: "1.0.0", channel: "unknown"))
        print("PASS update semantic ordering, preview channels and maintenance-release downgrade prevention")
    }
}
