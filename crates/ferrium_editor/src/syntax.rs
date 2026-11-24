use eframe::egui;
use eframe::egui::text::{LayoutJob, TextFormat};

const COL_DEFAULT: egui::Color32 = egui::Color32::from_rgb(208, 218, 236);
const COL_KEYWORD: egui::Color32 = egui::Color32::from_rgb(86, 156, 214);
const COL_CONTROL: egui::Color32 = egui::Color32::from_rgb(197, 134, 192);
const COL_TYPE: egui::Color32 = egui::Color32::from_rgb(78, 201, 176);
const COL_FN: egui::Color32 = egui::Color32::from_rgb(220, 220, 170);
const COL_NUMBER: egui::Color32 = egui::Color32::from_rgb(181, 206, 168);
const COL_STRING: egui::Color32 = egui::Color32::from_rgb(206, 145, 120);
const COL_COMMENT: egui::Color32 = egui::Color32::from_rgb(106, 153, 85);
const COL_BAKA: egui::Color32 = egui::Color32::from_rgb(244, 102, 128);
const COL_PURR: egui::Color32 = egui::Color32::from_rgb(122, 226, 240);
const COL_UWU: egui::Color32 = egui::Color32::from_rgb(242, 156, 216);

/// Syntax highlighter for FustyLang 0.2.1 (`nya`, `meow`, `uwu`, `baka!`, `purr`,
/// nested block comments `/* /* */ */`, and Ferrium host intrinsics).
pub fn highlight_fusty(source: &str, font_size: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font_id = egui::FontId::monospace(font_size);

    let push_seg = |job: &mut LayoutJob, text: &str, color: egui::Color32| {
        if text.is_empty() {
            return;
        }
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    };

    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        // Line comment
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            let start = i;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            push_seg(&mut job, &source[start..i], COL_COMMENT);
            continue;
        }

        // Nested block comment (FustyLang supports `/* outer /* inner */ outer */`)
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            let mut depth = 1usize;
            while i < len && depth > 0 {
                if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if i + 1 < len && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            push_seg(&mut job, &source[start..i], COL_COMMENT);
            continue;
        }

        // String literal
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            push_seg(&mut job, &source[start..i], COL_STRING);
            continue;
        }

        // Number literal
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'_') {
                i += 1;
            }
            push_seg(&mut job, &source[start..i], COL_NUMBER);
            continue;
        }

        // Identifier / keyword
        if is_ident_start(bytes[i]) {
            let start = i;
            while i < len && is_ident_continue(bytes[i]) {
                i += 1;
            }
            let word = &source[start..i];

            // Special FustyLang 0.2.1 `baka!(...)` panic macro-expression
            if word == "baka" && i < len && bytes[i] == b'!' {
                i += 1;
                push_seg(&mut job, &source[start..i], COL_BAKA);
                continue;
            }

            let next_is_call = source[i..].trim_start().starts_with('(');
            let color = match word {
                "nya" | "let" | "mut" | "struct" | "enum" | "impl" | "true" | "false" | "self"
                | "nil" => COL_KEYWORD,
                "if" | "else" | "while" | "for" | "in" | "return" | "match" | "wait" | "yield"
                | "meow" => COL_CONTROL,
                "uwu" => COL_UWU,
                "purr" if next_is_call => COL_PURR,
                "vec2" | "vec3" | "i32" | "i64" | "f32" | "f64" | "bool" | "string" => COL_TYPE,
                _ if next_is_call => COL_FN,
                _ => COL_DEFAULT,
            };
            push_seg(&mut job, word, color);
            continue;
        }

        // Punctuation / whitespace (preserve UTF-8 char boundaries)
        let ch = source[i..].chars().next().unwrap();
        let next = i + ch.len_utf8();
        push_seg(&mut job, &source[i..next], COL_DEFAULT);
        i = next;
    }

    job
}

#[inline]
fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

#[inline]
fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
