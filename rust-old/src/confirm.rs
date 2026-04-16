//! Interactive confirmation prompts.
//!
//! These are used by the CLI binary, not the library, keeping the library
//! free of stdin dependencies.

use std::io::{BufRead, Write};

/// Prompt the user for confirmation on stdout/stdin.
pub fn confirm(prompt: &str) -> bool {
    confirm_with(prompt, &mut std::io::stdin().lock(), &mut std::io::stdout())
}

/// Testable confirmation: reads from any `BufRead`, writes prompt to any `Write`.
pub fn confirm_with(prompt: &str, reader: &mut impl BufRead, writer: &mut impl Write) -> bool {
    let _ = write!(writer, "{} [y/N] ", prompt);
    let _ = writer.flush();
    let mut input = String::new();
    if reader.read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn confirm_yes() {
        let mut input = Cursor::new(b"y\n");
        let mut output = Vec::new();
        assert!(confirm_with("proceed?", &mut input, &mut output));
    }

    #[test]
    fn confirm_yes_full() {
        let mut input = Cursor::new(b"yes\n");
        let mut output = Vec::new();
        assert!(confirm_with("proceed?", &mut input, &mut output));
    }

    #[test]
    fn confirm_no() {
        let mut input = Cursor::new(b"n\n");
        let mut output = Vec::new();
        assert!(!confirm_with("proceed?", &mut input, &mut output));
    }

    #[test]
    fn confirm_empty_is_no() {
        let mut input = Cursor::new(b"\n");
        let mut output = Vec::new();
        assert!(!confirm_with("proceed?", &mut input, &mut output));
    }

    #[test]
    fn confirm_case_insensitive() {
        let mut input = Cursor::new(b"Y\n");
        let mut output = Vec::new();
        assert!(confirm_with("proceed?", &mut input, &mut output));
    }

    #[test]
    fn confirm_prompt_is_written() {
        let mut input = Cursor::new(b"n\n");
        let mut output = Vec::new();
        confirm_with("Delete all?", &mut input, &mut output);
        let written = String::from_utf8(output).unwrap();
        assert!(written.contains("Delete all?"));
    }
}
