using BibCiTeX.Core;

namespace BibCiTeX;

/// <summary>Editable tag structure; Rust owns validation and evaluation.</summary>
internal sealed class TagFilter
{
    internal string? TagId { get; set; }
    internal bool UsesOr { get; set; }
    internal bool Negated { get; set; }
    internal List<TagFilter> Children { get; } = [];
    internal bool IsEmpty => TagId is null && Children.Count == 0;
    internal bool IsComplete => TagId is not null || Children.Count > 0 && Children.All(child => child.IsComplete);
    internal IEnumerable<string> TagIds => TagId is { } id ? [id] : Children.SelectMany(child => child.TagIds);
    internal QueryDocumentRecord Document()
    {
        var nodes = new List<QueryNodeRecord>();
        uint Append(QueryNodeKind kind, string text = "", string? tagId = null, List<uint>? children = null)
        {
            var index = (uint)nodes.Count;
            nodes.Add(new() { kind = kind, text_value = text, tag_id = tagId, children = children ?? [], start = 0, end = 0 });
            return index;
        }
        uint Encode(TagFilter node)
        {
            uint index;
            if (node.TagId is { } id)
                index = Append(QueryNodeKind.Scope, "tag", children: [Append(QueryNodeKind.Word, "tag", id)]);
            else
                index = Append(node.UsesOr ? QueryNodeKind.Or : QueryNodeKind.And, children: node.Children.Select(Encode).ToList());
            return node.Negated ? Append(QueryNodeKind.Not, children: [index]) : index;
        }
        uint? root = IsEmpty ? null : Encode(this);
        return new() { nodes = nodes, root = root };
    }
    internal void ReplaceTagId(string oldId, string newId)
    {
        if (TagId == oldId) TagId = newId;
        foreach (var child in Children) child.ReplaceTagId(oldId, newId);
    }
    internal TagFilter Clone()
    {
        var copy = new TagFilter { TagId = TagId, UsesOr = UsesOr, Negated = Negated };
        copy.Children.AddRange(Children.Select(child => child.Clone()));
        return copy;
    }
    internal string Expression(IReadOnlyDictionary<string, string> names)
    {
        var value = TagId is { } id ? "\"" + names.GetValueOrDefault(id, "?").Replace("\\", "\\\\").Replace("\"", "\\\"") + "\""
            : "(" + string.Join(UsesOr ? " OR " : " AND ", Children.Select(child => child.Expression(names))) + ")";
        return (Negated ? "NOT " : "") + value;
    }
}
