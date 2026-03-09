use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use std::{
    io,
    time::Duration,
};

mod model;
mod tui;
mod remote;

use tui::{App, UI, ViewMode};

#[tokio::main]
async fn main() -> io::Result<()> {

    // Start remote server
    let remote_tx = remote::start_server(3000);

    // e to edit texts
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(remote_tx.clone());
    let mut remote_rx = remote_tx.subscribe();

    loop {
        app.update();
        terminal.draw(|f| UI::draw(f, &app))?;

        tokio::select! {
            result = async {
                if event::poll(Duration::from_millis(16))? {
                    if let Event::Key(key) = event::read()? {
                        return Ok::<Option<event::KeyEvent>, io::Error>(Some(key));
                    }
                }
                Ok(None)
            } => {
                if let Ok(Some(key)) = result {
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
            msg = remote_rx.recv() => {
                if let Ok(text) = msg {
                    if text == "Connected" || text == "Disconnected" || text.starts_with("Listening") {
                        app.server_status = text;
                    } else {
                        app.server_status = format!("Msg: {}", text);
                    }
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
