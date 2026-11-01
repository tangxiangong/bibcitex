import Foundation
import BibCiTeXCore

/// Editable tag structure; Rust owns validation and evaluation.
struct TagFilter: Identifiable, Equatable, Sendable {
    var id = UUID()
    var tagID: String?
    var usesOr = false
    var negated = false
    var children: [TagFilter] = []

    var isEmpty: Bool { tagID == nil && children.isEmpty }
    var tagIDs: Set<String> {
        if let tagID { return [tagID] }
        return children.reduce(into: Set<String>()) { $0.formUnion($1.tagIDs) }
    }
    var isComplete: Bool { tagID != nil || (!children.isEmpty && children.allSatisfy(\.isComplete)) }

    var document: QueryDocumentRecord {
        var nodes: [QueryNodeRecord] = []
        func append(_ kind: QueryNodeKind, text: String = "", tagID: String? = nil, children: [UInt32] = []) -> UInt32 {
            let index = UInt32(nodes.count)
            nodes.append(QueryNodeRecord(kind: kind, textValue: text, tagId: tagID, children: children, start: 0, end: 0))
            return index
        }
        func encode(_ node: TagFilter) -> UInt32 {
            let index: UInt32
            if let tagID = node.tagID {
                let leaf = append(.word, text: "tag", tagID: tagID)
                index = append(.scope, text: "tag", children: [leaf])
            } else {
                let children = node.children.map(encode)
                index = append(node.usesOr ? .or : .and, children: children)
            }
            return node.negated ? append(.not, children: [index]) : index
        }
        let root = isEmpty ? nil : encode(self)
        return QueryDocumentRecord(nodes: nodes, root: root)
    }

    func expression(names: [String: String]) -> String {
        let value: String
        if let tagID {
            value = names[tagID].map {
                "\"" + $0.replacingOccurrences(of: "\\", with: "\\\\")
                    .replacingOccurrences(of: "\"", with: "\\\"") + "\""
            } ?? "?"
        } else { value = "(" + children.map { $0.expression(names: names) }.joined(separator: usesOr ? " OR " : " AND ") + ")" }
        return negated ? "NOT " + value : value
    }

    mutating func replaceTagID(_ oldID: String, with newID: String) {
        if tagID == oldID { tagID = newID }
        for index in children.indices { children[index].replaceTagID(oldID, with: newID) }
    }

    func node(_ target: UUID) -> TagFilter? {
        if id == target { return self }
        return children.lazy.compactMap { $0.node(target) }.first
    }
    mutating func update(_ target: UUID, _ change: (inout TagFilter) -> Void) {
        if id == target { change(&self); return }
        for index in children.indices { children[index].update(target, change) }
    }
    mutating func remove(_ target: UUID) {
        children.removeAll { $0.id == target }
        for index in children.indices { children[index].remove(target) }
    }
}
