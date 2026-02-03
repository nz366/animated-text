use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line as TuiLine, Span},
    widgets::{Block, Borders, Paragraph},
};

use std::{
    io,
    time::{Duration, Instant},
};

mod model;
use model::{AnimationData, Keyframe, LyricLine};

#[derive(PartialEq)]
enum EditMode {
    Time,
    Progress,
}

#[derive(PartialEq)]
enum ViewMode {
    Focus,
    List,
    TextEdit,
}

struct App {
    data: AnimationData,
    current_time: f32,
    is_playing: bool,

    view_mode: ViewMode,
    edit_mode: EditMode,
    scroll_offset: usize,
    manual_scroll: bool,
    last_tick: Instant,

    focus_line_index: Option<usize>,
    active_kf_index: Option<usize>,
    cursor_col: usize,
    history: Vec<AnimationData>,
    history_index: usize,
}

impl App {
    fn new() -> Self {
        Self {
            data: AnimationData::demo(),
            current_time: 0.0,
            is_playing: false,
            view_mode: ViewMode::List,
            edit_mode: EditMode::Time,
            scroll_offset: 0,
            manual_scroll: false,
            last_tick: Instant::now(),
            focus_line_index: None,
            active_kf_index: None,
            cursor_col: 0,
            history: vec![], // You might want to push initial state here
            history_index: 0,
        }
    }

    // Helper to save state for UNDO
    fn push_history(&mut self) {
        // Remove any "redo" history if we branch off
        if self.history_index < self.history.len() {
            self.history.truncate(self.history_index);
        }
        // Clone current data (assuming AnimationData derives Clone)
        self.history.push(self.data.clone());
        self.history_index += 1;
    }

    fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(state) = self.history.get(self.history_index) {
                self.data = state.clone();
            }
        }
    }
    fn compile(&self) -> String {
        self.data.to_string()
    }

    fn update(&mut self) {
        let delta = self.last_tick.elapsed().as_secs_f32();
        self.last_tick = Instant::now();

        if self.is_playing {
            self.current_time += delta;

            // --- LOOPING LOGIC FOR FOCUS MODE ---
            if self.view_mode == ViewMode::Focus {
                // If we have a tracked line index, use it to check boundaries
                if let Some(idx) = self.focus_line_index {
                    let line = &self.data.lines[idx];

                    // If we pass the end of the line, loop back to start
                    if self.current_time > line.end {
                        self.current_time = line.start;
                    }
                    // Safety: If we manually seeked way before the line, snap to start
                    else if self.current_time < line.start {
                        self.current_time = line.start;
                    }
                } else {
                    // If no line is tracked (e.g., startup), try to find one
                    self.focus_line_index = self.get_active_line_index();
                }
            } else {
                // In LIST MODE, we clear the focus lock so playback flows normally
                self.focus_line_index = None;

                // if all linse are read seek back to first line
            }
        }

        // --- SCROLL SYNC LOGIC ---
        if !self.manual_scroll {
            if let Some(idx) = self.get_active_line_index() {
                self.scroll_offset = idx;
                // Update focus index if we are just entering focus mode or drifting naturally
                if self.view_mode == ViewMode::Focus && self.focus_line_index.is_none() {
                    self.focus_line_index = Some(idx);
                }
            } else {
                let closest = self
                    .data
                    .lines
                    .iter()
                    .position(|l| self.current_time >= l.start)
                    .unwrap_or(0);
                self.scroll_offset = closest;
            }
        }
    }

    fn get_active_line_index(&self) -> Option<usize> {
        // Find line that currently contains the time
        self.data
            .lines
            .iter()
            .position(|l| self.current_time >= l.start && self.current_time <= l.end)
    }

    fn handle_control_input(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }

        match key.code {
            KeyCode::Char(' ') => self.is_playing = !self.is_playing,
            KeyCode::Char('e') => {
                // 1. Switch mode
                self.view_mode = ViewMode::TextEdit;

                // 2. Ensure we have a line to edit.
                // If nothing is selected via scroll, use the playing line or line 0.
                if self.focus_line_index.is_none() {
                    self.focus_line_index = Some(self.scroll_offset);
                }

                // 3. Initialize cursor position to the end of the string
                if let Some(idx) = self.focus_line_index {
                    self.cursor_col = self.data.lines[idx].text.chars().count();
                }
            }
            // Seeking logic
            KeyCode::Left => {
                self.active_kf_index = None;
                self.current_time = (self.current_time - 0.5).max(0.0);
                // If we seek in focus mode, update the locked line to the new time
                if self.view_mode == ViewMode::Focus {
                    self.focus_line_index = self.get_active_line_index();
                }
            }
            KeyCode::Right => {
                self.active_kf_index = None;
                self.current_time += 0.5;
                if self.view_mode == ViewMode::Focus {
                    self.focus_line_index = self.get_active_line_index();
                }
            }

            KeyCode::Esc => self.toggle_view_mode(),

            // Navigate lines in Focus Mode (Prev/Next Line)
            // This is useful because normal playback loops, so we need keys to force change line
            KeyCode::Char('n') if self.view_mode == ViewMode::Focus => {
                if let Some(curr) = self.focus_line_index {
                    if curr + 1 < self.data.lines.len() {
                        self.focus_line_index = Some(curr + 1);
                        self.current_time = self.data.lines[curr + 1].start;
                    }
                }
            }
            KeyCode::Char('p') if self.view_mode == ViewMode::Focus => {
                if let Some(curr) = self.focus_line_index {
                    if curr > 0 {
                        self.focus_line_index = Some(curr - 1);
                        self.current_time = self.data.lines[curr - 1].start;
                    }
                }
            }

            KeyCode::Char('s') => {
                let _ = self.compile();
            }

            KeyCode::PageUp if self.view_mode == ViewMode::List => self.seek_list(-1),
            KeyCode::PageDown if self.view_mode == ViewMode::List => self.seek_list(1),
            // KeyCode::Esc if self.view_mode == ViewMode::List => self.manual_scroll = false,
            _ => {
                if self.view_mode == ViewMode::Focus {
                    self.handle_keyframe_editor_keys(key.code);
                }
            }
        }
    }

    fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Focus => {
                // When switching TO List, unlock the line focus so song plays through
                self.focus_line_index = None;
                ViewMode::List
            }
            ViewMode::List => {
                self.manual_scroll = false;
                self.focus_line_index = self.get_active_line_index();
                self.active_kf_index = None;
                ViewMode::Focus
            }
            ViewMode::TextEdit => {
                self.focus_line_index = None;
                ViewMode::List
            }
        };
    }

    fn scroll_list(&mut self, dir: i32) {
        self.manual_scroll = true;
        let len = self.data.lines.len();
        if len == 0 {
            return;
        }

        let new_idx = (self.scroll_offset as i32 + dir).clamp(0, (len - 1) as i32);
        self.scroll_offset = new_idx as usize;
    }

    fn seek_list(&mut self, dir: i32) {
        self.manual_scroll = true;
        let len = self.data.lines.len();
        if len == 0 {
            return;
        }

        // Compute new index and clamp
        let new_idx = (self.scroll_offset as i32 + dir).clamp(0, (len - 1) as i32);
        self.scroll_offset = new_idx as usize;

        // SEEK the player to the line's start time
        self.current_time = self.data.lines[self.scroll_offset].start;

        // Clear focus so we don't interfere with playback
        self.focus_line_index = None;
        self.active_kf_index = None;
    }
    fn handle_keyframe_editor_keys(&mut self, code: KeyCode) {
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
                self.data.lines[idx].keyframes.push(Keyframe {
                    time: rel_time,
                    index: target_idx,
                });
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
                        self.data.lines[idx].keyframes[ki].index =
                            (self.data.lines[idx].keyframes[ki].index + 0.5 * mult)
                                .clamp(0.0, line_len);
                    }
                    EditMode::Time => {
                        let line = &mut self.data.lines[idx];
                        let rel_time = (line.keyframes[ki].time + 0.05 * mult).max(0.0);
                        line.keyframes[ki].time = rel_time;

                        let is_boundary = ki == line.keyframes.len() - 1;

                        if is_boundary {
                            line.end = (line.start + rel_time).max(line.start + 0.01);
                            line.keyframes[ki].time = line.end - line.start;
                        }
                    }
                }
            }

            KeyCode::Char('k') => {
                let line = &self.data.lines[idx];
                let rel_time = self.current_time - line.start;

                // 1) Try next keyframe
                if let Some((i, kf)) = line
                    .keyframes
                    .iter()
                    .enumerate()
                    .find(|(_, k)| k.time > rel_time + 0.01)
                {
                    self.active_kf_index = Some(i);
                    self.current_time = line.start + kf.time;
                    return;
                }

                // 2) Otherwise jump to next line
                if idx + 1 < self.data.lines.len() {
                    let next_idx = idx + 1;
                    let next_line = &self.data.lines[next_idx];

                    self.focus_line_index = Some(next_idx);
                    self.current_time = next_line.start;
                    self.active_kf_index = Some(0); // first KF
                }
                self.is_playing = false;
            }

            KeyCode::Char('j') => {
                let line = &self.data.lines[idx];
                let rel_time = self.current_time - line.start;

                // 1) Try previous keyframe
                if let Some((i, kf)) = line
                    .keyframes
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, k)| k.time < rel_time - 0.01)
                {
                    self.active_kf_index = Some(i);
                    self.current_time = line.start + kf.time;
                    return;
                }

                // 2) Otherwise jump to previous line
                if idx > 0 {
                    let prev_idx = idx - 1;
                    let prev_line = &self.data.lines[prev_idx];

                    self.focus_line_index = Some(prev_idx);
                    self.current_time = prev_line.start;
                    self.active_kf_index = Some(prev_line.keyframes.len().saturating_sub(1)); // last KF
                }
                self.is_playing = false;
            }

            _ => {}
        }
    }

    fn find_closest_kf_idx(&self, line_idx: usize, rel_time: f32) -> Option<usize> {
        self.data.lines[line_idx]
            .keyframes
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.time - rel_time)
                    .abs()
                    .partial_cmp(&(b.time - rel_time).abs())
                    .unwrap()
            })
            .map(|(i, _)| i)
    }

    fn handle_text_edits(&mut self, key: KeyEvent) {
        use crossterm::event::KeyModifiers;
        if key.kind == KeyEventKind::Release {
            return;
        }
        if key.code == KeyCode::Esc {
            self.toggle_view_mode();
            return;
        }

        // Ensure we have a valid line selected
        let line_idx = if let Some(i) = self.focus_line_index {
            i
        } else {
            return;
        };

        match key.code {
            // --- NAVIGATION ---
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
                    // MOVE LINE UP
                    if line_idx > 0 {
                        self.push_history(); // Save state
                        self.data.lines.swap(line_idx, line_idx - 1);
                        self.focus_line_index = Some(line_idx - 1);
                    }
                } else {
                    // NAVIGATE UP
                    if line_idx > 0 {
                        self.focus_line_index = Some(line_idx - 1);
                        // Clamp cursor to new line length
                        let new_len = self.data.lines[line_idx - 1].text.chars().count();
                        self.cursor_col = self.cursor_col.min(new_len);
                    }
                }
            }
            KeyCode::Down => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    // MOVE LINE DOWN
                    if line_idx + 1 < self.data.lines.len() {
                        self.push_history(); // Save state
                        self.data.lines.swap(line_idx, line_idx + 1);
                        self.focus_line_index = Some(line_idx + 1);
                    }
                } else {
                    // NAVIGATE DOWN
                    if line_idx + 1 < self.data.lines.len() {
                        self.focus_line_index = Some(line_idx + 1);
                        // Clamp cursor to new line length
                        let new_len = self.data.lines[line_idx + 1].text.chars().count();
                        self.cursor_col = self.cursor_col.min(new_len);
                    }
                }
            }

            // --- EDITING ---
            KeyCode::Char(c) => {
                // Determine byte index from char index to insert correctly
                let mut current_text: Vec<char> = self.data.lines[line_idx].text.chars().collect();
                current_text.insert(self.cursor_col, c);
                self.data.lines[line_idx].text = current_text.into_iter().collect();
                self.cursor_col += 1;
            }

            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    // Delete char within current line
                    let mut current_text: Vec<char> =
                        self.data.lines[line_idx].text.chars().collect();
                    current_text.remove(self.cursor_col - 1);
                    self.data.lines[line_idx].text = current_text.into_iter().collect();
                    self.cursor_col -= 1;
                } else if line_idx > 0 {
                    // MERGE with previous line
                    self.push_history();

                    let curr_text = self.data.lines[line_idx].text.clone();
                    // Remove current line
                    self.data.lines.remove(line_idx);

                    // Append text to previous line
                    let prev_idx = line_idx - 1;
                    let prev_len = self.data.lines[prev_idx].text.chars().count();
                    self.data.lines[prev_idx].text.push_str(&curr_text);

                    // Update Focus
                    self.focus_line_index = Some(prev_idx);
                    self.cursor_col = prev_len;
                }
            }

            KeyCode::Enter => {
                self.push_history();
                // Split the string
                let chars: Vec<char> = self.data.lines[line_idx].text.chars().collect();
                let (left, right) = chars.split_at(self.cursor_col);

                let left_str: String = left.iter().collect();
                let right_str: String = right.iter().collect();

                // Update current line
                self.data.lines[line_idx].text = left_str;

                // Create new line
                // Logic: New line starts where old line ends (roughly)
                let old_end = self.data.lines[line_idx].end;
                let new_line = LyricLine {
                    part: None,
                    text: right_str,
                    start: old_end,
                    end: old_end + 2.0, // Arbitrary duration for new line
                    keyframes: vec![],
                };

                self.data.lines.insert(line_idx + 1, new_line);

                // Move focus
                self.focus_line_index = Some(line_idx + 1);
                self.cursor_col = 0;
            }

            // --- UNDO ---
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.undo();
            }

            _ => {}
        }
    }
}

// --- UI RENDERING ---

struct UI;

impl UI {
    fn draw(f: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(f.size());

        Self::render_header(f, app, chunks[0]);

        match app.view_mode {
            ViewMode::Focus => Self::render_focus_mode(f, app, chunks[1]),
            ViewMode::List => Self::render_list_mode(f, app, chunks[1]),
            ViewMode::TextEdit => Self::render_list_mode(f, app, chunks[1]),
        }
    }

    fn render_header(f: &mut Frame, app: &App, area: Rect) {
        let mode_str = match app.view_mode {
            ViewMode::List => "LINE MODE [ESC] | TEXT EDIT [E] | [Q] Quit | [SPACE] Play", // Updated indicator
            ViewMode::Focus => "FULL MODE [ESC] | [Q] Quit | [SPACE] Play",
            ViewMode::TextEdit => "DONE [ESC]",
        };

        let status_color = if app.view_mode == ViewMode::List || app.view_mode == ViewMode::TextEdit
        {
            Color::Blue
        } else if app.is_playing {
            Color::Green
        } else {
            Color::Yellow
        };
        let rel_time = app
            .focus_line_index
            .or(app.get_active_line_index())
            .map(|idx| {
                let line = &app.data.lines[idx];
                (app.current_time - line.start).clamp(0.0, line.end - line.start)
            })
            .unwrap_or(0.0);

        let info = format!(
            " {} | Time: {:.2}s |  Relative: {:.2}s ",
            mode_str, app.current_time, rel_time
        );

        let sub_info = if app.view_mode == ViewMode::List && app.manual_scroll {
            " MANUAL SCROLLING (Press ESC to Auto)".to_string()
        } else if app.view_mode == ViewMode::Focus {
            " [N] Next Line | [P] Prev Line".to_string()
        } else {
            " ".to_string()
        };

        let p = Paragraph::new(vec![TuiLine::from(info), TuiLine::from(sub_info)]).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(status_color)),
        );

        f.render_widget(p, area);
    }

    fn render_focus_mode(f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(5)])
            .split(area);

        // Use focus_line_index if available, otherwise fall back to time-based lookup
        let active_idx = app.focus_line_index.or(app.get_active_line_index());

        if let Some(idx) = active_idx {
            Self::render_active_line_anim(f, app, idx, chunks[0]);
            Self::render_keyframe_editor_panel(f, app, idx, chunks[1]);
        } else {
            f.render_widget(
                Paragraph::new("Waiting for next line...").alignment(Alignment::Center),
                chunks[0],
            );
        }
    }

    fn get_animated_line_spans<'a>(
        line: &'a LyricLine,
        current_time: f32,
        is_active: bool,
    ) -> Vec<Span<'a>> {
        let rel_time = current_time - line.start;
        let target_idx = line.get_current_index(rel_time);

        line.text
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let mut style = Style::default();

                if is_active {
                    let mut color = Color::Rgb(255, 255, 255); // Default "played" color (White)

                    if target_idx < i as f32 {
                        // "Unplayed" or "Glow" logic
                        let dist = (i as f32 - target_idx).abs();
                        let intensity = (1.0 - (dist / 2.5)).clamp(0.0, 1.0);
                        color = Color::Rgb(
                            (60.0 + 195.0 * intensity) as u8,
                            (60.0 + 195.0 * intensity) as u8,
                            (60.0 + 40.0 * (1.0 - intensity)) as u8,
                        );
                    }
                    style = style.fg(color).add_modifier(Modifier::BOLD);
                } else {
                    style = style.fg(Color::DarkGray);
                }

                Span::styled(c.to_string(), style)
            })
            .collect()
    }

    fn render_active_line_anim(f: &mut Frame, app: &App, idx: usize, area: Rect) {
        let line = &app.data.lines[idx];
        let spans = Self::get_animated_line_spans(line, app.current_time, true);

        f.render_widget(
            Paragraph::new(TuiLine::from(spans))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE)),
            area,
        );
    }

    fn render_list_mode(f: &mut Frame, app: &App, area: Rect) {
        let active_idx = app.get_active_line_index();
        let display_idx = app.scroll_offset;
        let is_text_editor = app.view_mode == ViewMode::TextEdit;

        let mut tui_lines = Vec::new();

        for (i, lyric) in app.data.lines.iter().enumerate() {
            let is_playing = Some(i) == active_idx;
            let is_editing = is_text_editor && app.focus_line_index == Some(i);
            let is_selected = (app.manual_scroll && i == display_idx) || is_editing;

            let prefix = if is_playing {
                " >> "
            } else if is_selected {
                " -> "
            } else {
                "    "
            };

            // 1. Time and Prefix Spans
            let mut line_spans = vec![
                Span::styled(
                    format!("[{:.2}] ", lyric.start),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    prefix,
                    if is_playing {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Blue)
                    },
                ),
            ];

            // 2. Render Text Content
            if is_editing {
                // Text Edit Mode: Render with Cursor
                let text_chars: Vec<char> = lyric.text.chars().collect();
                for (char_idx, c) in text_chars.iter().enumerate() {
                    let char_style = if char_idx == app.cursor_col {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    line_spans.push(Span::styled(c.to_string(), char_style));
                }
                if app.cursor_col >= text_chars.len() {
                    line_spans.push(Span::styled(" ", Style::default().bg(Color::Blue)));
                }
            } else if is_playing {
                let animated_content =
                    Self::get_animated_line_spans(lyric, app.current_time, is_playing);
                line_spans.extend(animated_content);
            } else {
                // Standard Idle Line
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                line_spans.push(Span::styled(lyric.text.clone(), style));
            }

            tui_lines.push(TuiLine::from(line_spans));
        }

        // Scroll logic (unchanged)
        let scroll_target = if is_text_editor {
            app.focus_line_index.unwrap_or(display_idx)
        } else {
            display_idx
        };
        let scroll_pos = if scroll_target > 5 {
            (scroll_target - 5) as u16
        } else {
            0
        };

        let title = if is_text_editor {
            " Exit Edit [ESC] "
        } else {
            " [E] EDIT "
        };

        let p = Paragraph::new(tui_lines)
            .alignment(Alignment::Left)
            .scroll((scroll_pos, 0))
            .block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(p, area);
    }

    fn render_keyframe_editor_panel(f: &mut Frame, app: &App, idx: usize, area: Rect) {
        let line = &app.data.lines[idx];
        let rel_time = app.current_time - line.start;

        let kfs = line
            .keyframes
            .iter()
            .enumerate()
            .map(|(ki, k)| {
                let is_near = (k.time - rel_time).abs() < 0.1;
                Span::styled(
                    format!(
                        " [KF{}: {:.2}s|{:.0}%] ",
                        ki,
                        k.time,
                        (k.index / line.text.len().max(1) as f32) * 100.0
                    ),
                    Style::default().fg(if is_near {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    }),
                )
            })
            .collect::<Vec<_>>();

        let mode_str = if app.edit_mode == EditMode::Time {
            "EDIT: TIME"
        } else {
            "EDIT: PROGRESS"
        };

        let ui_info = vec![
            TuiLine::from(kfs),
            TuiLine::from(Span::styled(
                format!(" LINE {} | {}", idx + 1, mode_str),
                Style::default().bg(Color::Cyan).fg(Color::Black),
            )),
            TuiLine::from(" [T] Toggle Edit Mode | [F] Add | [G/Del] Delete"),
            TuiLine::from(" [J/K] Jump | [UP/DOWN] Adjust Value"),
        ];

        f.render_widget(
            Paragraph::new(ui_info)
                .block(Block::default().borders(Borders::TOP).title("Editor"))
                .alignment(Alignment::Center),
            area,
        );
    }
}

fn main() -> io::Result<()> {
    // e to edit texts
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        app.update();
        terminal.draw(|f| UI::draw(f, &app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if app.view_mode == ViewMode::TextEdit {
                    app.handle_text_edits(key);
                } else {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                    app.handle_control_input(key);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    let data = app.compile();
    print!("{}", data);
    Ok(())
}
