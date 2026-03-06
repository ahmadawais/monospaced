pub fn to_monospace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for ch in text.chars() {
        let mapped = match ch {
            'A'..='Z' => std::char::from_u32(0x1d670 + (ch as u32 - 'A' as u32)).unwrap_or(ch),
            'a'..='z' => std::char::from_u32(0x1d68a + (ch as u32 - 'a' as u32)).unwrap_or(ch),
            '0'..='9' => std::char::from_u32(0x1d7f6 + (ch as u32 - '0' as u32)).unwrap_or(ch),
            _ => ch,
        };

        result.push(mapped);
    }

    result
}
