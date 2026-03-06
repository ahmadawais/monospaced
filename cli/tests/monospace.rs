use monospaced::to_monospace;

#[test]
fn converts_letters_and_digits() {
    assert_eq!(to_monospace("npx 14"), "𝚗𝚙𝚡 𝟷𝟺");
}

#[test]
fn preserves_punctuation_and_whitespace() {
    assert_eq!(to_monospace("hello, world!\n"), "𝚑𝚎𝚕𝚕𝚘, 𝚠𝚘𝚛𝚕𝚍!\n");
}

#[test]
fn leaves_unsupported_characters_untouched() {
    assert_eq!(to_monospace("é Ω _ -"), "é Ω _ -");
}
