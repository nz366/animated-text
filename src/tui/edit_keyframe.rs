use crate::model::{Keyframe, TextSegment};
use crate::tui::app::App;
use crate::tui::edit_line::UI;
use crate::tui::types::EditMode;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line as TuiLine, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

impl App {
    pub fn handle_keyframe_editor_keys(&mut self, code: KeyCode) {
        let Some(idx) = self.focus_line_index.or(self.get_active_line_index()) else {
            return;
        };
        let rel_time = self.current_time - self.data.lines[idx].start;
        let line_len = self.data.lines[idx].text.len() as f32;

        match code {
            KeyCode::Char('t') => {
                self.edit_mode = match self.edit_mode {
                    EditMode::Progress => EditMode::Time,
                    EditMode::Time => EditMode::Progress,
                }
            }
            KeyCode::Char('f') => {
                let target_idx = self.data.lines[idx].get_current_index(rel_time);

                if self.data.lines[idx].keyframes.is_empty() {
                    let (start, end, text_len) = {
                        let line = &self.data.lines[idx];
                        (line.start, line.end, line.text.len() as f32)
                    };

                    self.data.lines[idx].keyframes.push(Keyframe {
                        time: 0.0,
                        index: 0.0,
                    });

                    if idx + 1 < self.data.lines.len() {
                        let next_start = self.data.lines[idx + 1].start;
                        self.data.lines[idx].keyframes.push(Keyframe {
                            time: next_start - start,
                            index: text_len,
                        });
                    } else {
                        self.data.lines[idx].keyframes.push(Keyframe {
                            time: end - start,
                            index: text_len,
                        });
                    }
                } else {
                    self.data.lines[idx].keyframes.push(Keyframe {
                        time: rel_time,
                        index: target_idx,
                    });
                }

                self.data.lines[idx].sort_keyframes();
            }
            KeyCode::Char('g') | KeyCode::Delete => {
                if self.data.lines[idx].keyframes.len() > 1 {
                    if let Some(ki) = self.find_closest_kf_idx(idx, rel_time) {
                        self.data.lines[idx].keyframes.remove(ki);
                    }
                }
            }
            KeyCode::Up | KeyCode::Down => {
                let Some(ki) = self.active_kf_index else {
                    return;
                };

                let mult = if code == KeyCode::Up { 1.0 } else { -1.0 };

                match self.edit_mode {
                    EditMode::Progress => {
                        self.data.edit_keyframe_position(idx, ki, 0.5 * mult);
                    }
                    EditMode::Time => {
                        self.data.edit_keyframe_time(idx, ki, 0.5 * mult);
                    }
                }
            }

            KeyCode::Char('k') => {
                let line = &self.data.lines[idx];
                let rel_time = self.current_time - line.start;

                if let Some((i, kf)) = line
                    .keyframes
                    .iter()
                    .enumerate()
                    .find(|(_, k)| k.time > rel_time + 0.01)
                {
                    self.active_kf_index = Some(i);
                    self.set_time(line.start + kf.time);
                    return;
                }

                if idx + 1 < self.data.lines.len() {
                    let next_idx = idx + 1;
                    let next_line = &self.data.lines[next_idx];

                    self.focus_line_index = Some(next_idx);
                    self.set_time(next_line.start);
                    self.active_kf_index = Some(0);
                }
                self.is_playing = false;
            }

            KeyCode::Char('j') => {
                let line = &self.data.lines[idx];
                let rel_time = self.current_time - line.start;

                if let Some((i, kf)) = line
                    .keyframes
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, k)| k.time < rel_time - 0.01)
                {
                    self.active_kf_index = Some(i);
                    self.set_time(line.start + kf.time);
                    return;
                }

                if idx > 0 {
                    let prev_idx = idx - 1;
                    let prev_line = &self.data.lines[prev_idx];

                    self.focus_line_index = Some(prev_idx);

                    let last_idx = prev_line.keyframes.len().saturating_sub(1);
                    self.active_kf_index = Some(last_idx);

                    if let Some(last_kf) = prev_line.keyframes.get(last_idx) {
                        self.set_time(prev_line.start + last_kf.time);
                    } else {
                        self.set_time(prev_line.start);
                    }
                }
                self.is_playing = false;
            }
            _ => {}
        }
    }
}

impl UI {
    pub fn render_keyframe_editor_panel(f: &mut Frame, app: &App, idx: usize, area: Rect) {
        let line = &app.data.lines[idx];
        let rel_time = app.current_time - line.start;

        let kfs = line
            .keyframes
            .iter()
            .enumerate()
            .flat_map(|(ki, k)| {
                let is_near = if let Some(aki) = app.active_kf_index {
                    ki == aki
                } else {
                    (k.time - rel_time).abs() < 0.1
                };

                let base_fg = if is_near {
                    Color::Yellow
                } else {
                    Color::DarkGray
                };

                let time_style = if app.edit_mode == EditMode::Time && is_near {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default().fg(base_fg)
                };

                let progress_style = if app.edit_mode == EditMode::Progress && is_near {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default().fg(base_fg)
                };

                vec![
                    Span::styled(format!(" [KF{}: ", ki), Style::default().fg(base_fg)),
                    Span::styled(format!("{:7.2}s", k.time), time_style),
                    Span::styled("|", Style::default().fg(base_fg)),
                    Span::styled(
                        format!("{:.0}%", (k.index / line.text.len().max(1) as f32) * 100.0),
                        progress_style,
                    ),
                    Span::styled("] ", Style::default().fg(base_fg)),
                ]
            })
            .collect::<Vec<Span>>();

        let toggle_edit = if app.edit_mode == EditMode::Time {
            "Edit Position"
        } else {
            "Edit Time"
        };

        let ui_info = vec![
            TuiLine::from(kfs),
            TuiLine::from(format!(
                " [T] {} | [F] Add | [G/Del] Delete | [J/K] Jump | [UP/DOWN] Adjust Value",
                toggle_edit
            )),
        ];

        f.render_widget(
            Paragraph::new(ui_info)
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .title("KeyFrame Editor"),
                )
                .alignment(Alignment::Center),
            area,
        );
    }

    pub fn gen_animated_line_spans<'a>(
        line: &'a TextSegment,
        current_time: f32,
        is_active: bool,
    ) -> Vec<Span<'a>> {
        if !is_active {
            return vec![Span::raw(line.text.clone())];
        }

        let rel_time = current_time - line.start;
        let target_idx = line.get_current_index(rel_time);

        line.text
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let mut style = Style::default();

                let mut color = Color::Rgb(255, 255, 255);

                if target_idx < i as f32 {
                    let dist = (i as f32 - target_idx).abs();
                    let intensity = (1.0 - (dist / 2.5)).clamp(0.0, 1.0);
                    color = Color::Rgb(
                        (60.0 + 195.0 * intensity) as u8,
                        (60.0 + 195.0 * intensity) as u8,
                        (60.0 + 40.0 * (1.0 - intensity)) as u8,
                    );
                }
                style = style.fg(color).add_modifier(Modifier::BOLD);

                Span::styled(c.to_string(), style)
            })
            .collect()
    }

    pub fn render_active_line_anim(f: &mut Frame, app: &App, idx: usize, area: Rect) {
        let line = &app.data.lines[idx];
        let spans = Self::gen_animated_line_spans(line, app.current_time, true);

        f.render_widget(
            Paragraph::new(TuiLine::from(spans))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE)),
            area,
        );
    }
}
