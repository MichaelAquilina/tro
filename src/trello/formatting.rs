pub fn title(text: &str) -> String {
    let border = "═".repeat(text.chars().count());

    [
        format!("╔═{}═╗", border),
        format!("║ {} ║", text),
        format!("╚═{}═╝", border),
    ]
    .join("\n")
}

pub fn header(text: &str, header_char: &str) -> String {
    [text, &header_char.repeat(text.chars().count())].join("\n")
}
