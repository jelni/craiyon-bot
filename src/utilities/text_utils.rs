use std::fmt::{self, Write};

pub const MARKDOWN_CHARS: [char; 20] = [
    '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!', '`',
    '\\',
];

pub trait TruncateWithEllipsis {
    fn truncate_with_ellipsis(self, max_len: usize) -> Self;
}

impl TruncateWithEllipsis for String {
    fn truncate_with_ellipsis(mut self, max_len: usize) -> Self {
        if self.chars().count() > max_len {
            self.truncate(max_len - 1);
            self.push('…');
        }

        self
    }
}

pub struct EscapeMarkdown<'a>(pub &'a str);

impl fmt::Display for EscapeMarkdown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.0.chars() {
            if MARKDOWN_CHARS.contains(&ch) {
                f.write_char('\\')?;
            }
            f.write_char(ch)?;
        }

        Ok(())
    }
}

pub fn format_duration(duration: u64) -> String {
    let hours = (duration / 3600) % 60;
    let minutes = (duration / 60) % 60;
    let seconds = duration % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

pub fn progress_bar(current: u32, max: u32) -> String {
    #[allow(clippy::cast_precision_loss)]
    let step = max as f32 / 20.;
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]
    let char_count = (current as f32 / step) as usize;

    let mut progress = String::with_capacity(22);
    progress.push('[');
    progress.push_str(&"=".repeat(char_count));
    progress.push_str(&"-".repeat(20 - char_count));
    progress.push(']');

    progress
}

pub fn check_prompt(prompt: &str) -> Option<&'static str> {
    if prompt.chars().count() > 512 {
        Some("this prompt is too long (>512).")
    } else if prompt.lines().count() > 4 {
        Some("this prompt has too many lines (>4).")
    } else {
        None
    }
}
