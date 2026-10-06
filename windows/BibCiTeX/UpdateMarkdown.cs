using Markdig;
using Markdig.Syntax;
using Markdig.Extensions.Tables;
using Markdig.Syntax.Inlines;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Documents;

namespace BibCiTeX;

/// Markdown is rendered as native text; HTML and remote images are never executed or loaded.
internal static class UpdateMarkdown
{
    private static readonly MarkdownPipeline Pipeline = new MarkdownPipelineBuilder().UsePipeTables().Build();
    internal static UIElement Render(string markdown)
    {
        var panel = new StackPanel { Spacing = 10 };
        AddBlocks(panel, Markdown.Parse(markdown, Pipeline));
        return panel;
    }
    private static void AddBlocks(StackPanel panel, ContainerBlock blocks)
    {
        foreach (var block in blocks)
        {
            if (block is HtmlBlock) continue;
            if (block is CodeBlock code)
            {
                panel.Children.Add(new TextBlock { Text = code.Lines.ToString(), FontFamily = new Microsoft.UI.Xaml.Media.FontFamily("Consolas"), TextWrapping = TextWrapping.Wrap, IsTextSelectionEnabled = true });
            }
            else if (block is LeafBlock leaf)
            {
                var text = new TextBlock { TextWrapping = TextWrapping.Wrap, IsTextSelectionEnabled = true };
                if (block is HeadingBlock heading) { text.FontSize = Math.Max(15, 25 - heading.Level * 2); text.FontWeight = FontWeights.SemiBold; }
                if (leaf.Inline is { } inline) AddInline(text.Inlines, inline);
                panel.Children.Add(text);
            }
            else if (block is ListBlock list)
            {
                var number = 1;
                foreach (var item in list.OfType<ContainerBlock>())
                {
                    var row = new Grid { ColumnSpacing = 8 };
                    row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
                    row.ColumnDefinitions.Add(new ColumnDefinition());
                    row.Children.Add(new TextBlock { Text = list.IsOrdered ? $"{number++}." : "•" });
                    var content = new StackPanel { Spacing = 4 }; AddBlocks(content, item); Grid.SetColumn(content, 1); row.Children.Add(content); panel.Children.Add(row);
                }
            }
            else if (block is Table table)
            {
                var grid = new Grid { ColumnSpacing = 12, RowSpacing = 6 };
                for (var c = 0; c < table.ColumnDefinitions.Count; c++) grid.ColumnDefinitions.Add(new ColumnDefinition());
                var rowIndex = 0;
                foreach (var row in table.OfType<TableRow>())
                {
                    grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
                    var column = 0;
                    foreach (var cell in row.OfType<TableCell>())
                    {
                        var body = new StackPanel { Spacing = 4 }; AddBlocks(body, cell);
                        Grid.SetRow(body, rowIndex); Grid.SetColumn(body, column++); grid.Children.Add(body);
                    }
                    rowIndex++;
                }
                panel.Children.Add(grid);
            }
            else if (block is ContainerBlock container) AddBlocks(panel, container);
        }
    }
    private static void AddInline(InlineCollection output, ContainerInline input)
    {
        foreach (var item in input)
        {
            switch (item)
            {
                case LiteralInline literal: output.Add(new Run { Text = literal.Content.ToString() }); break;
                case CodeInline code: output.Add(new Run { Text = code.Content, FontFamily = new Microsoft.UI.Xaml.Media.FontFamily("Consolas") }); break;
                case LineBreakInline: output.Add(new LineBreak()); break;
                case EmphasisInline emphasis:
                    Span span = emphasis.DelimiterCount >= 2 ? new Bold() : new Italic(); AddInline(span.Inlines, emphasis); output.Add(span); break;
                case LinkInline link when !link.IsImage:
                    if (Uri.TryCreate(link.Url, UriKind.Absolute, out var uri) && uri.Scheme is "https" or "http")
                    { var anchor = new Hyperlink { NavigateUri = uri }; AddInline(anchor.Inlines, link); output.Add(anchor); }
                    else AddInline(output, link);
                    break;
                case ContainerInline children when item is not LinkInline: AddInline(output, children); break;
            }
        }
    }
}
