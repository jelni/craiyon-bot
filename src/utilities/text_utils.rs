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

pub fn escape_markdown<S: AsRef<str>>(text: S) -> String {
    let text = text.as_ref();
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if MARKDOWN_CHARS.contains(&ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
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

pub fn check_prompt<S: AsRef<str>>(prompt: S) -> Option<&'static str> {
    let prompt = prompt.as_ref();
    if prompt.chars().count() > 512 {
        Some("this prompt is too long (>512).")
    } else if prompt.lines().count() > 4 {
        Some("this prompt has too many lines (>4).")
    } else {
        None
    }
}
