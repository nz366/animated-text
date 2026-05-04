use crate::model::TextSegment;
use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

impl App {
    pub fn handle_text_edits(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        if key.code == KeyCode::Esc {
            self.process_bracket_parts();
            self.data.add_trailing_empty();
            self.focus_line_index = Some(self.data.lines.len() - 1);
            self.cursor_col = 0;
            self.toggle_view_mode();
            return;
        }

        let line_idx = if let Some(i) = self.focus_line_index {
            i
        } else {
            return;
        };

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if key.code == KeyCode::Char('z') {
                self.undo();
                return;
            }
            if key.code == KeyCode::Char('v') {
                match arboard::Clipboard::new() {
                    Ok(mut cb) => match cb.get_text() {
                        Ok(text) => self.insert_text(&text),
                        Err(e) => self.server_status = format!("Paste err: {}", e),
                    },
                    Err(e) => self.server_status = format!("Cb init err: {}", e),
                }
                return;
            }
        }

        match key.code {
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            KeyCode::Right => {
                let line_len = self.data.lines[line_idx].text.chars().count();
                if self.cursor_col < line_len {
                    self.cursor_col += 1;
                }
            }
            KeyCode::Up => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    if line_idx > 0 {
                        self.push_history();
                        self.data.lines.swap(line_idx, line_idx - 1);
                        self.focus_line_index = Some(line_idx - 1);
                    }
                } else {
                    if line_idx > 0 {
                        self.focus_line_index = Some(line_idx - 1);
                        let new_len = self.data.lines[line_idx - 1].text.chars().count();
                        self.cursor_col = self.cursor_col.min(new_len);
                    }
                }
            }
            KeyCode::Down => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    if line_idx + 1 < self.data.lines.len() {
                        self.push_history();
                        self.data.lines.swap(line_idx, line_idx + 1);
                        self.focus_line_index = Some(line_idx + 1);
                    }
                } else {
                    if line_idx + 1 < self.data.lines.len() {
                        self.focus_line_index = Some(line_idx + 1);
                        let new_len = self.data.lines[line_idx + 1].text.chars().count();
                        self.cursor_col = self.cursor_col.min(new_len);
                    }
                }
            }
            KeyCode::Char(c) => {
                let mut current_text: Vec<char> = self.data.lines[line_idx].text.chars().collect();
                current_text.insert(self.cursor_col, c);
                self.data.lines[line_idx].text = current_text.into_iter().collect();
                self.cursor_col += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    let mut current_text: Vec<char> =
                        self.data.lines[line_idx].text.chars().collect();
                    current_text.remove(self.cursor_col - 1);
                    self.data.lines[line_idx].text = current_text.into_iter().collect();
                    self.cursor_col -= 1;
                } else if line_idx > 0 {
                    self.push_history();

                    let curr_text = self.data.lines[line_idx].text.clone();
                    self.data.lines.remove(line_idx);

                    let prev_idx = line_idx - 1;
                    let prev_len = self.data.lines[prev_idx].text.chars().count();
                    self.data.lines[prev_idx].text.push_str(&curr_text);

                    self.focus_line_index = Some(prev_idx);
                    self.cursor_col = prev_len;
                }
            }
            KeyCode::Enter => {
                self.push_history();
                let chars: Vec<char> = self.data.lines[line_idx].text.chars().collect();
                let (left, right) = chars.split_at(self.cursor_col);

                let left_str: String = left.iter().collect();
                let right_str: String = right.iter().collect();

                self.data.lines[line_idx].text = left_str;

                let old_end = self.data.lines[line_idx].end;
                let new_line = TextSegment {
                    part: None,
                    text: right_str,
                    start: old_end,
                    end: old_end + 2.0,
                    keyframes: vec![],
                };

                self.data.lines.insert(line_idx + 1, new_line);

                self.focus_line_index = Some(line_idx + 1);
                self.cursor_col = 0;
            }
            _ => {}
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let line_idx = if let Some(i) = self.focus_line_index {
            i
        } else {
            return;
        };

        self.push_history();

        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            // Handle case where text might be just newlines
            let newline_count = text.chars().filter(|&c| c == '\n').count();
            if newline_count > 0 {
                for _ in 0..newline_count {
                    self.handle_text_edits(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
                }
            }
            return;
        }

        if lines.len() == 1 && !text.ends_with('\n') {
            // Single line paste
            let mut current_text: Vec<char> = self.data.lines[line_idx].text.chars().collect();
            let paste_chars: Vec<char> = lines[0].chars().collect();
            for (i, c) in paste_chars.iter().enumerate() {
                current_text.insert(self.cursor_col + i, *c);
            }
            self.data.lines[line_idx].text = current_text.into_iter().collect();
            self.cursor_col += paste_chars.len();
        } else {
            // Multi-line paste
            let current_line_text: Vec<char> = self.data.lines[line_idx].text.chars().collect();
            let (prefix, suffix) = current_line_text.split_at(self.cursor_col);
            let prefix_str: String = prefix.iter().collect();
            let suffix_str: String = suffix.iter().collect();

            // First line gets prefix + first pasted line
            self.data.lines[line_idx].text = format!("{}{}", prefix_str, lines[0]);

            // "Minute automatic filling": each line gets 60s and they are sequential
            let first_line_start = self.data.lines[line_idx].start;
            self.data.lines[line_idx].end = first_line_start + 60.0;
            let mut last_end = self.data.lines[line_idx].end;

            let mut current_idx = line_idx;

            // Intermediate lines
            for i in 1..lines.len() - 1 {
                let new_line = TextSegment {
                    part: None,
                    text: lines[i].to_string(),
                    start: last_end,
                    end: last_end + 60.0,
                    keyframes: vec![],
                };
                self.data.lines.insert(current_idx + 1, new_line);
                current_idx += 1;
                last_end += 60.0;
            }

            // Last line gets last pasted line + suffix
            let last_pasted = lines.last().unwrap();
            let new_line = TextSegment {
                part: None,
                text: format!("{}{}", last_pasted, suffix_str),
                start: last_end,
                end: last_end + 60.0,
                keyframes: vec![],
            };
            self.data.lines.insert(current_idx + 1, new_line);
            current_idx += 1;

            self.focus_line_index = Some(current_idx);
            self.cursor_col = last_pasted.chars().count();
        }
    }

    pub fn process_bracket_parts(&mut self) {
        let mut i = 0;
        while i < self.data.lines.len() {
            let trimmed = self.data.lines[i].text.trim().to_string();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let part = trimmed;
                self.data.lines.remove(i);
                if i < self.data.lines.len() {
                    self.data.lines[i].part = Some(part);
                }
            } else {
                i += 1;
            }
        }
    }
}
