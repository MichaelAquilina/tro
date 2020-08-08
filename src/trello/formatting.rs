use unicode_width::UnicodeWidthStr;

pub fn title(text: &str) -> String {
    let border = "=".repeat(UnicodeWidthStr::width(text));

    [
        format!(" {} ", text),
        format!("={}=", border),
    ]
    .join("\n")
}

pub fn header(text: &str, header_char: &str) -> String {
    [text, &header_char.repeat(UnicodeWidthStr::width(text))].join("\n")
}
