use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line as TuiLine, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::App;
use crate::tui::types::ViewMode;
pub struct UI;

impl UI {
    pub fn draw(f: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(f.area());

        Self::render_header(f, app, chunks[0]);

        match app.view_mode {
            ViewMode::Line => Self::render_focus_mode(f, app, chunks[1]),
            ViewMode::List => Self::render_list_mode(f, app, chunks[1]),
            ViewMode::TextEdit => Self::render_list_mode(f, app, chunks[1]),
            ViewMode::DraftSelector => Self::render_draft_selector(f, app, chunks[1]),
        }
    }

    fn render_header(f: &mut Frame, app: &App, area: Rect) {
        let mode_str = match app.view_mode {
            ViewMode::List => {
                "LINE MODE [ESC] | TEXT EDIT [E] | KEYFRAME EDIT [J]  | [Q] Quit | [SPACE] Play"
            }
            ViewMode::Line => "LIST MODE [ESC] | [Q] Quit | [SPACE] Play",
            ViewMode::TextEdit => "DONE [ESC] | [SHIFT+INSERT] Paste",
            ViewMode::DraftSelector => "SELECT DRAFT [UP/DOWN] | LOAD [ENTER] | NEW [N]",
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
            " {} | Time: {:7.2}s |  Relative: {:7.2}s ",
            mode_str, app.current_time, rel_time
        );

        let sub_info = if app.view_mode == ViewMode::List && app.manual_scroll {
            " MANUAL SCROLLING (Press ESC to Auto)".to_string()
        } else if app.view_mode == ViewMode::Line {
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

            let mut line_spans = vec![
                Span::styled(
                    format!("[{:7.2}] ", lyric.start),
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

            if is_editing {
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
                    Self::gen_animated_line_spans(lyric, app.current_time, is_playing);
                line_spans.extend(animated_content);
            } else {
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
            " Editing ".to_string()
        } else {
            format!(" [E] EDIT | Status: {} ", app.server_status)
        };

        let p = Paragraph::new(tui_lines)
            .alignment(Alignment::Left)
            .scroll((scroll_pos, 0))
            .block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(p, area);
    }
}
