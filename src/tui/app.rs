use crate::model::AnimationData;
use crate::tui::types::{EditMode, ViewMode};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::time::Instant;

pub struct App {
    pub data: AnimationData,
    pub current_time: f32,
    pub is_playing: bool,

    pub view_mode: ViewMode,
    pub edit_mode: EditMode,
    pub scroll_offset: usize,
    pub manual_scroll: bool,
    pub last_tick: Instant,

    pub focus_line_index: Option<usize>,
    pub active_kf_index: Option<usize>,
    pub cursor_col: usize,
    pub history: Vec<AnimationData>,
    pub history_index: usize,
    pub remote_tx: tokio::sync::broadcast::Sender<String>,
    pub server_status: String,

    pub drafts: Vec<String>,
    pub selected_draft: usize,
}

impl App {
    pub fn new(remote_tx: tokio::sync::broadcast::Sender<String>) -> Self {
        let mut drafts = Vec::new();
        if let Ok(entries) = std::fs::read_dir("drafts") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.ends_with(".txt") {
                        drafts.push(name);
                    }
                }
            }
        }
        drafts.sort();

        Self {
            data: AnimationData::new(),
            current_time: 0.0,
            is_playing: false,
            view_mode: ViewMode::DraftSelector,
            edit_mode: EditMode::Time,
            scroll_offset: 0,
            manual_scroll: false,
            last_tick: Instant::now(),
            focus_line_index: None,
            active_kf_index: None,
            cursor_col: 0,
            history: vec![],
            history_index: 0,
            remote_tx,
            server_status: "Listening...".to_string(),
            drafts,
            selected_draft: 0,
        }
    }

    pub fn push_history(&mut self) {
        if self.history_index < self.history.len() {
            self.history.truncate(self.history_index);
        }
        self.history.push(self.data.clone());
        self.history_index += 1;
    }

    pub fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(state) = self.history.get(self.history_index) {
                self.data = state.clone();
            }
        }
    }

    pub fn load_draft(&mut self, filename: &str) {
        let path = format!("drafts/{}", filename);
        match std::fs::read_to_string(path) {
            Ok(content) => match content.parse::<AnimationData>() {
                Ok(data) => {
                    self.data = data;
                    self.view_mode = ViewMode::List;
                    self.server_status = format!("Loaded {}", filename);
                }
                Err(e) => {
                    self.server_status = format!("Failed to parse draft: {}", e);
                }
            },
            Err(e) => {
                self.server_status = format!("Failed to read draft: {}", e);
            }
        }
    }

    pub fn update_time(&mut self, time: f32) {
        self.current_time = time;
    }

    pub fn set_time(&mut self, time: f32) {
        self.current_time = time;
        if self
            .remote_tx
            .send(format!(
                r#"{{"command": "seek", "time": {}}}"#,
                self.current_time
            ))
            .is_err()
        {
            self.server_status = "Sending failed".to_string();
        }
    }

    pub fn update(&mut self) {
        let delta = self.last_tick.elapsed().as_secs_f32();
        self.last_tick = Instant::now();

        if self.is_playing {
            self.update_time(self.current_time + delta);

            if self.view_mode == ViewMode::Line {
                if let Some(idx) = self.focus_line_index {
                    let line = &self.data.lines[idx];

                    if self.current_time > line.end {
                        self.set_time(line.start);
                    } else if self.current_time < line.start {
                        self.set_time(line.start);
                    }
                } else {
                    self.focus_line_index = self.get_active_line_index();
                }
            } else {
                self.focus_line_index = None;

                if let Some(last_line) = self.data.lines.last() {
                    // if self.current_time > last_line.end {
                    //     self.is_playing = false;
                    //     let _ = self.remote_tx.send("pause".to_string());
                    // }
                }
            }
        }

        if !self.manual_scroll {
            if let Some(idx) = self.get_active_line_index() {
                self.scroll_offset = idx;
                if self.view_mode == ViewMode::Line && self.focus_line_index.is_none() {
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

    pub fn get_active_line_index(&self) -> Option<usize> {
        self.data
            .lines
            .iter()
            .position(|l| self.current_time >= l.start && self.current_time <= l.end)
    }

    pub fn handle_control_input(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }

        // Global keys — apply in all non-DraftSelector modes
        if self.view_mode != ViewMode::DraftSelector {
            match key.code {
                KeyCode::Char('s') => {
                    let _ = self.data.compile();
                }
                KeyCode::Char(' ') => {
                    self.is_playing = !self.is_playing;

                    if let Some(last_line) = self.data.lines.last() {
                        if self.current_time > last_line.end {
                            self.set_time(0.0);
                            self.is_playing = false;
                            self.server_status = "EOF reached".to_string();
                        }
                    }

                    let cmd = if self.is_playing { "play" } else { "pause" };
                    let _ = self.remote_tx.send(cmd.to_string());
                }
                KeyCode::Char('e') => {
                    self.view_mode = ViewMode::TextEdit;
                    if self.focus_line_index.is_none() {
                        self.focus_line_index = Some(self.scroll_offset);
                    }
                    if let Some(idx) = self.focus_line_index {
                        self.cursor_col = self.data.lines[idx].text.chars().count();
                    }
                }
                KeyCode::Char('j') => {
                    self.manual_scroll = false;
                    self.focus_line_index = self.get_active_line_index();
                    self.active_kf_index = None;
                    self.view_mode = ViewMode::Line;
                }
                KeyCode::Left => {
                    self.active_kf_index = None;
                    self.set_time((self.current_time - 0.5).max(0.0));
                    if self.view_mode == ViewMode::Line {
                        self.focus_line_index = self.get_active_line_index();
                    }
                }
                KeyCode::Right => {
                    self.active_kf_index = None;
                    self.set_time(self.current_time + 0.5);
                    if self.view_mode == ViewMode::Line {
                        self.focus_line_index = self.get_active_line_index();
                    }
                }
                KeyCode::Esc => self.toggle_view_mode(),
                _ => {}
            }
        }

        // Mode-specific keys
        match self.view_mode {
            ViewMode::DraftSelector => match key.code {
                KeyCode::Up => {
                    if self.selected_draft > 0 {
                        self.selected_draft -= 1;
                    }
                }
                KeyCode::Down => {
                    if !self.drafts.is_empty() && self.selected_draft < self.drafts.len() - 1 {
                        self.selected_draft += 1;
                    }
                }
                KeyCode::Enter => {
                    if !self.drafts.is_empty() {
                        let filename = self.drafts[self.selected_draft].clone();
                        self.load_draft(&filename);
                    } else {
                        self.view_mode = ViewMode::List;
                    }
                }
                KeyCode::Char('n') => {
                    self.data = AnimationData::new();
                    self.view_mode = ViewMode::List;
                }

                KeyCode::Esc => self.toggle_view_mode(),
                _ => {}
            },

            ViewMode::Line => match key.code {
                KeyCode::Char('n') => {
                    if let Some(curr) = self.focus_line_index {
                        if curr + 1 < self.data.lines.len() {
                            self.focus_line_index = Some(curr + 1);
                            self.set_time(self.data.lines[curr + 1].start);
                        }
                    }
                }
                KeyCode::Char('p') => {
                    if let Some(curr) = self.focus_line_index {
                        if curr > 0 {
                            self.focus_line_index = Some(curr - 1);
                            self.set_time(self.data.lines[curr - 1].start);
                        }
                    }
                }
                _ => {
                    self.handle_keyframe_editor_keys(key.code);
                }
            },

            ViewMode::List => match key.code {
                KeyCode::Up => {
                    self.manual_scroll = true;
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
                KeyCode::Down => {
                    self.manual_scroll = true;
                    if self.scroll_offset < self.data.lines.len() - 1 {
                        self.scroll_offset += 1;
                    }
                }
                KeyCode::PageUp => self.seek_list(-1),
                KeyCode::PageDown => self.seek_list(1),
                KeyCode::End => {
                    // set end time stamp of previous line to current_time
                    if let Some(idx) = self.get_active_line_index() {
                        self.push_history();
                        let duration = self.data.lines[idx].start - self.current_time;
                        self.data.lines[idx].end = self.current_time +  duration;
                    }q
                }
                KeyCode::Home => {
                    if self.scroll_offset < self.data.lines.len() {
                        self.push_history();
                        let current_time = self.current_time;
                        {
                            let idx = self.scroll_offset;

                            let (before, current_and_after) = self.data.lines.split_at_mut(idx);
                            let line = &mut current_and_after[0];

                            let duration = line.end - line.start;
                            line.start = current_time;
                            line.end = current_time + duration;

                            if idx > 0 {
                                let prev_line = &mut before[idx - 1];
                                if prev_line.keyframes.len() <= 2 {
                                    prev_line.end = line.start - 0.01;
                                }
                            }
                        }


                        if (self.scroll_offset < self.data.lines.len() - 1) {
                            if(self.data.lines[self.scroll_offset + 1].text.trim().is_empty()){
                                if(self.scroll_offset < self.data.lines.len()-2){
                                    self.scroll_offset += 2;
                                }else{
                                    self.scroll_offset += 1;
                                }
                            }else{
                                self.scroll_offset += 1;
                            }
                        } else {
                            self.scroll_offset = 0;
                        }

                        self.set_time(current_time);
                    }
                }
                _ => {}
            },

            _ => {}
        }
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Line => {
                self.focus_line_index = None;
                ViewMode::List
            }
            ViewMode::List => {
                self.manual_scroll = false;
                self.focus_line_index = self.get_active_line_index();
                self.active_kf_index = None;
                // ViewMode::Line
                ViewMode::DraftSelector
            }
            ViewMode::TextEdit => {
                self.focus_line_index = None;
                ViewMode::List
            }
            ViewMode::DraftSelector => ViewMode::List,
        };
    }

    pub fn seek_list(&mut self, dir: i32) {
        self.manual_scroll = false;
        let len = self.data.lines.len();
        if len == 0 {
            return;
        }

        let new_idx = (self.scroll_offset as i32 + dir).clamp(0, (len - 1) as i32);
        self.set_time(self.data.lines[new_idx as usize].start);
        self.focus_line_index = None;
        self.active_kf_index = None;
    }

    pub fn find_closest_kf_idx(&self, line_idx: usize, rel_time: f32) -> Option<usize> {
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
}
