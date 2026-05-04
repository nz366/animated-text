use crate::tui::app::App;
use crate::tui::edit_line::UI;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line as TuiLine, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

impl UI {
    pub fn render_draft_selector(f: &mut Frame, app: &App, area: Rect) {
        let mut tui_lines = Vec::new();

        if app.drafts.is_empty() {
            tui_lines.push(TuiLine::from(vec![Span::styled(
                " No drafts found in drafts/ folder ",
                Style::default().fg(Color::Gray),
            )]));
            tui_lines.push(TuiLine::from(vec![Span::styled(
                " Press [N] to start a new project ",
                Style::default().fg(Color::Yellow),
            )]));
        } else {
            for (i, draft) in app.drafts.iter().enumerate() {
                let is_selected = i == app.selected_draft;
                let prefix = if is_selected { " -> " } else { "    " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                tui_lines.push(TuiLine::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Blue)),
                    Span::styled(draft, style),
                ]));
            }
        }

        let p = Paragraph::new(tui_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Drafts Selector | Status: {}", app.server_status)),
        );

        f.render_widget(p, area);
    }
}
