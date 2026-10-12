/// Tolerate exporters that double the slash before a multi-letter TeX command.
/// Row-bearing constructs are ambiguous: `\\alpha` can start a new row whose
/// contents are the letters "alpha", so preserve their original TeX semantics.
pub(super) fn normalize_commands(text: String) -> String {
    if !text.contains(r"\\")
        || [
            r"\begin",
            r"\substack",
            r"\subarray",
            r"\displaylines",
            r"\cases",
        ]
        .iter()
        .any(|command| text.contains(command))
    {
        return text;
    }

    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        let mut slashes = 1;
        while chars.next_if_eq(&'\\').is_some() {
            slashes += 1;
        }
        // A lone letter after `\\` is commonly the first cell of another row.
        if slashes == 2 && chars.clone().take_while(char::is_ascii_alphabetic).count() > 1 {
            slashes = 1;
        }
        result.extend(std::iter::repeat_n('\\', slashes));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::normalize_commands;

    #[test]
    fn doubled_commands_work_with_standard_commands_and_unicode() {
        assert_eq!(
            normalize_commands(r"\frac{\\ell}{2} + \\alpha + 文献".into()),
            r"\frac{\ell}{2} + \alpha + 文献"
        );
    }

    #[test]
    fn line_breaks_escapes_and_longer_slash_runs_are_preserved() {
        let source = r"a\\b + a\\ b + \\ + \\% + \\{x\\} + \\\ell + \\\\alpha";
        assert_eq!(normalize_commands(source.into()), source);
    }

    #[test]
    fn matrix_rows_starting_with_letters_keep_their_meaning() {
        let source = r"\begin{matrix}a & b\\alpha & beta\\\ell & c\end{matrix}";
        assert_eq!(normalize_commands(source.into()), source);
    }

    #[test]
    fn substack_rows_keep_their_meaning() {
        let source = r"\sum_{\substack{a\\beta}} x";
        assert_eq!(normalize_commands(source.into()), source);
    }
}
