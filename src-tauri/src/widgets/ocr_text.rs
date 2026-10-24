//! Turn OCR words (with positions) into text for each mode. Pure: no Windows calls.
use serde::{Deserialize, Serialize};

pub use crate::platform::Word;
pub type Line = Vec<Word>;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode { #[default] Text, Code, Terminal }

fn join(line: &Line) -> String { line.iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ") }

/// Median character width over all words (fallback 8 px).
fn char_width(lines: &[Line]) -> f32 {
    let mut v: Vec<f32> = lines.iter().flatten()
        .filter(|w| !w.text.is_empty())
        .map(|w| w.w / w.text.chars().count() as f32)
        .collect();
    if v.is_empty() { return 8.0; }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2].max(1.0)
}

/// Rebuild indentation from each line's left edge relative to the leftmost line.
fn indented(lines: &[Line]) -> Vec<String> {
    let cw = char_width(lines);
    let left = lines.iter().filter_map(|l| l.first()).map(|w| w.x).fold(f32::INFINITY, f32::min);
    lines.iter().map(|l| match l.first() {
        Some(w) => format!("{}{}", " ".repeat(((w.x - left) / cw).round().max(0.0) as usize), join(l)),
        None => String::new(),
    }).collect()
}

/// Remove a shell prompt: "PS C:\x> ", "user@host:~$ ", "$ ", "# ", "> ", "C:\x>".
pub fn strip_prompt(s: &str) -> &str {
    let t = s.trim_start();
    if t.starts_with("PS ") { if let Some(i) = t.find("> ") { return &t[i + 2..]; } }
    if let Some(i) = t.find("$ ") { if t[..i].contains('@') || i == 0 { return &t[i + 2..]; } }
    for p in ["# ", "> ", "% "] { if let Some(r) = t.strip_prefix(p) { return r; } }
    // cmd.exe: "C:\path>command"
    let b = t.as_bytes();
    if b.len() > 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\' {
        if let Some(i) = t.find('>') { return t[i + 1..].trim_start(); }
    }
    t
}

pub fn format(lines: &[Line], mode: Mode) -> String {
    match mode {
        Mode::Text => lines.iter().map(join).collect::<Vec<_>>().join("\n"),
        Mode::Code => indented(lines).join("\n"),
        Mode::Terminal => lines.iter().map(|l| strip_prompt(&join(l)).to_string()).collect::<Vec<_>>().join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn w(t: &str, x: f32) -> Word { Word { text: t.into(), x, w: t.len() as f32 * 10.0 } }

    #[test]
    fn text_joins_words_and_lines() {
        let l = vec![vec![w("hello", 0.0), w("world", 60.0)], vec![w("bye", 0.0)]];
        assert_eq!(format(&l, Mode::Text), "hello world\nbye");
    }

    #[test]
    fn code_restores_indentation() {
        let l = vec![vec![w("fn", 20.0), w("main()", 50.0)], vec![w("body();", 60.0)], vec![w("}", 20.0)]];
        assert_eq!(format(&l, Mode::Code), "fn main()\n    body();\n}");
    }

    #[test]
    fn terminal_strips_prompts() {
        assert_eq!(strip_prompt("PS C:\\Users\\me> git status"), "git status");
        assert_eq!(strip_prompt("me@box:~/src$ ls -la"), "ls -la");
        assert_eq!(strip_prompt("$ npm i"), "npm i");
        assert_eq!(strip_prompt("C:\\Windows>dir"), "dir");
        assert_eq!(strip_prompt("echo $ HOME"), "echo $ HOME");
        assert_eq!(strip_prompt("plain output"), "plain output");
    }
}
